//! Applet state and the update loop.
//!
//! The model is deliberately small: a fresh [`HardwareSnapshot`] each tick, a
//! little history for the charts, and which tab the user is looking at.

use std::time::Duration;

use cosmic::app::{Core, Task};
use cosmic::cosmic_config::{Config, CosmicConfigEntry};
use cosmic::iced::window::Id;
use cosmic::iced::Subscription;
use cosmic::{Application, Element};

use crate::config::{TempTyleConfig, ThemePreference, APP_ID, CONFIG_VERSION};
use crate::hardware::types::{HardwareSnapshot, TemperatureUnit};
use crate::hardware::HardwareCollector;
use crate::views;

/// Samples kept per chart. At the default two-second refresh this is roughly
/// the last four minutes.
const HISTORY_LEN: usize = 120;

/// Which detail view the popup is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveTab {
    #[default]
    Overview,
    CpuDetail,
    GpuDetail,
    MemoryDetail,
    StorageDetail,
    Settings,
}

/// A rolling window of recent samples backing one sparkline.
///
/// Oldest samples fall off the front once it is full, so the chart always
/// shows the most recent [`HISTORY_LEN`] readings.
#[derive(Default)]
pub struct History {
    samples: Vec<f32>,
}

impl History {
    fn push(&mut self, value: f32) {
        if self.samples.len() >= HISTORY_LEN {
            // Shifting 120 floats a few times a minute costs nothing, and
            // keeps the samples contiguous for the chart to borrow.
            self.samples.remove(0);
        }

        self.samples.push(value);
    }

    pub fn as_slice(&self) -> &[f32] {
        &self.samples
    }
}

pub struct AppModel {
    pub core: Core,
    pub popup: Option<Id>,
    pub config: TempTyleConfig,
    pub unit: TemperatureUnit,
    pub active_tab: ActiveTab,
    /// Whether the overview shows its expanded detail sections.
    pub expanded_details: bool,
    /// Whether the CPU tab shows the per-core grid.
    pub expanded_cores: bool,
    pub collector: HardwareCollector,
    pub snapshot: HardwareSnapshot,
    pub cpu_temp: History,
    pub gpu_temp: History,
    pub memory_usage: History,
}

#[derive(Clone, Debug)]
pub enum Message {
    /// The refresh timer fired.
    Tick,
    SelectTab(ActiveTab),
    ToggleExpandedDetails,
    ToggleCpuCores,
    SetTheme(ThemePreference),
    SetUnit(TemperatureUnit),
    SetInterval(u64),
    ClosePopup,
    PopupClosed(Id),
    Surface(cosmic::surface::Action),
    /// The config changed, possibly in another instance of the applet.
    ConfigChanged(TempTyleConfig),
}

impl AppModel {
    /// Whether to draw against a dark background, honouring an explicit
    /// preference over the desktop's current theme.
    pub fn is_dark(&self) -> bool {
        match self.config.theme_pref {
            ThemePreference::Dark => true,
            ThemePreference::Light => false,
            ThemePreference::System => cosmic::theme::is_dark(),
        }
    }

    /// Takes a reading and appends it to the charts.
    fn refresh(&mut self) {
        self.snapshot = self.collector.collect();
        self.record();
    }

    /// Appends the current snapshot to the charts.
    fn record(&mut self) {
        self.cpu_temp.push(self.snapshot.cpu.package_temp);
        self.memory_usage.push(self.snapshot.memory.percent);
        if let Some(gpu) = &self.snapshot.gpu {
            self.gpu_temp.push(gpu.edge_temp);
        }
    }

    /// Persists the config. Failure is not worth interrupting the user for:
    /// the change still applies to the running applet.
    fn save_config(&self) {
        if let Ok(handler) = Config::new(APP_ID, CONFIG_VERSION) {
            let _ = self.config.write_entry(&handler);
        }
    }

    fn set_unit(&mut self, unit: TemperatureUnit) {
        self.unit = unit;
        self.config.fahrenheit = unit == TemperatureUnit::Fahrenheit;
        self.save_config();
    }

    /// Asks the shell to re-theme the applet, unless it already follows the
    /// system theme and needs no override.
    fn apply_theme(pref: ThemePreference) -> Task<Message> {
        let Some(theme) = pref.theme() else {
            return Task::none();
        };
        cosmic::task::message(cosmic::Action::Cosmic(cosmic::app::Action::AppThemeChange(theme)))
    }
}

impl Application for AppModel {
    type Executor = cosmic::iced::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        // Start from the persisted config, falling back to defaults.
        let config = Config::new(APP_ID, CONFIG_VERSION)
            .ok()
            .and_then(|handler| TempTyleConfig::get_entry(&handler).ok())
            .unwrap_or_default();

        let mut collector = HardwareCollector::new();
        let mut app = Self {
            core,
            popup: None,
            unit: config.unit(),
            config,
            active_tab: ActiveTab::Overview,
            expanded_details: false,
            expanded_cores: false,
            snapshot: collector.collect(),
            collector,
            cpu_temp: History::default(),
            gpu_temp: History::default(),
            memory_usage: History::default(),
        };

        // Seed the charts so they have something to draw immediately.
        app.record();

        let theme = Self::apply_theme(app.config.theme_pref);
        (app, theme)
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Tick => self.refresh(),
            Message::SelectTab(tab) => self.active_tab = tab,
            Message::ToggleExpandedDetails => self.expanded_details = !self.expanded_details,
            Message::ToggleCpuCores => self.expanded_cores = !self.expanded_cores,
            Message::SetUnit(unit) => self.set_unit(unit),
            Message::SetInterval(seconds) => {
                self.config.refresh_interval_secs = seconds;
                self.save_config();
            }
            Message::SetTheme(pref) => {
                self.config.theme_pref = pref;
                self.save_config();
                return Self::apply_theme(pref);
            }
            Message::ConfigChanged(config) => {
                let theme_changed = config.theme_pref != self.config.theme_pref;
                self.unit = config.unit();
                self.config = config;
                if theme_changed {
                    return Self::apply_theme(self.config.theme_pref);
                }
            }
            Message::ClosePopup => {
                if let Some(id) = self.popup.take() {
                    return views::panel::destroy(id);
                }
            }
            Message::PopupClosed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
            }
            Message::Surface(action) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(action),
                ));
            }
        }

        Task::none()
    }

    /// The panel button: a thermometer tinted by thermal state, plus the
    /// current temperature.
    fn view(&self) -> Element<'_, Self::Message> {
        views::panel::view(self)
    }

    /// The popup opened by clicking the panel button.
    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        views::panel::popup_container(self, views::view_popup(self))
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let interval = Duration::from_secs(self.config.refresh_interval_secs.max(1));

        Subscription::batch([
            cosmic::iced::time::every(interval).map(|_| Message::Tick),
            // Picks up changes made by another instance of the applet.
            self.core()
                .watch_config::<TempTyleConfig>(APP_ID)
                .map(|update| Message::ConfigChanged(update.config)),
        ])
    }

    fn on_close_requested(&self, id: Id) -> Option<Self::Message> {
        Some(Message::PopupClosed(id))
    }
}
