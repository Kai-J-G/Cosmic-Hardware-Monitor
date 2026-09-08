use std::sync::LazyLock;
use std::time::Duration;

use cosmic::app::{Core, Task};
use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic::iced::window::Id;
use cosmic::iced::{Alignment, Length, Rectangle, Subscription};
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::widget;
use cosmic::{Application, Element};

use crate::config::{TempTyleConfig, APP_ID};
use crate::hardware::types::{HardwareSnapshot, TemperatureUnit};
use crate::hardware::HardwareCollector;
use crate::views;

static AUTOSIZE_MAIN_ID: LazyLock<widget::Id> = LazyLock::new(|| widget::Id::new("autosize-main"));
static AUTOSIZE_POPUP_ID: LazyLock<widget::Id> = LazyLock::new(|| widget::Id::new("cosmic-applet-autosize"));

fn get_cosmic_theme(pref: crate::config::ThemePreference) -> cosmic::Theme {
    let mut theme = match pref {
        crate::config::ThemePreference::Dark => cosmic::theme::system_dark(),
        crate::config::ThemePreference::Light => cosmic::theme::system_light(),
        crate::config::ThemePreference::System => cosmic::theme::system_preference(),
    };
    theme.transparent = true;
    theme
}

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

pub struct AppModel {
    pub core: Core,
    pub popup: Option<Id>,
    pub active_tab: ActiveTab,
    pub expanded_details: bool,
    pub unit: TemperatureUnit,
    pub collector: HardwareCollector,
    pub snapshot: HardwareSnapshot,
    pub history: Vec<f32>,
    pub cpu_load_history: Vec<f32>,
    pub gpu_temp_history: Vec<f32>,
    pub gpu_load_history: Vec<f32>,
    pub mem_history: Vec<f32>,
    pub max_history: usize,
    pub config: TempTyleConfig,
    pub expanded_cpu_cores: bool,
}

impl AppModel {
    pub fn is_dark(&self) -> bool {
        match self.config.theme_pref {
            crate::config::ThemePreference::Dark => true,
            crate::config::ThemePreference::Light => false,
            crate::config::ThemePreference::System => cosmic::theme::is_dark(),
        }
    }

    fn popup_container<'a>(
        &self,
        content: impl Into<Element<'a, Message>>,
        width: f32,
    ) -> Element<'a, Message> {
        use cosmic::applet::cosmic_panel_config::PanelAnchor;
        use cosmic::iced::alignment::{Horizontal, Vertical};

        let (vertical_align, horizontal_align) = match self.core.applet.anchor {
            PanelAnchor::Left => (Vertical::Center, Horizontal::Left),
            PanelAnchor::Right => (Vertical::Center, Horizontal::Right),
            PanelAnchor::Top => (Vertical::Top, Horizontal::Center),
            PanelAnchor::Bottom => (Vertical::Bottom, Horizontal::Center),
        };

        cosmic::widget::autosize::autosize(
            cosmic::widget::container(
                cosmic::widget::container(content).style(|theme| {
                    let cosmic = theme.cosmic();
                    let corners = cosmic.corner_radii;
                    let bg = cosmic.background(true).base;
                    cosmic::iced::widget::container::Style {
                        text_color: Some(cosmic.background(true).on.into()),
                        background: Some(cosmic::iced::Color::from(bg).into()),
                        border: cosmic::iced::Border {
                            radius: corners.radius_m.into(),
                            width: 1.0,
                            color: cosmic.background(true).divider.into(),
                        },
                        shadow: cosmic::iced::Shadow::default(),
                        icon_color: Some(cosmic.background(true).on.into()),
                        snap: true,
                    }
                }),
            )
            .height(Length::Shrink)
            .align_x(horizontal_align)
            .align_y(vertical_align),
            AUTOSIZE_POPUP_ID.clone(),
        )
        .limits(
            cosmic::iced::Limits::NONE
                .min_height(1.0)
                .min_width(width)
                .max_width(width)
                .max_height(1400.0),
        )
        .into()
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Tick,
    #[allow(dead_code)]
    Refresh,
    SelectTab(ActiveTab),
    ToggleExpandedDetails,
    ToggleCpuCores,
    ToggleUnits,
    SetTheme(crate::config::ThemePreference),
    SetUnit(TemperatureUnit),
    SetInterval(u64),
    ClosePopup,
    PopupClosed(Id),
    Surface(cosmic::surface::Action),
    ConfigChanged(TempTyleConfig),
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
        let mut collector = HardwareCollector::new();
        let snapshot = collector.collect();
        let initial_temp = snapshot.summary.package_temp;
        let initial_cpu_load = snapshot.cpu.overall_usage;
        let initial_gpu_temp = snapshot.gpu.as_ref().map(|g| g.edge_temp).unwrap_or(30.0);
        let initial_gpu_load = snapshot.gpu.as_ref().map(|g| g.utilization_percent as f32).unwrap_or(0.0);

        let mut history = Vec::with_capacity(120);
        history.push(initial_temp);

        let mut cpu_load_history = Vec::with_capacity(120);
        cpu_load_history.push(initial_cpu_load);

        let mut gpu_temp_history = Vec::with_capacity(120);
        gpu_temp_history.push(initial_gpu_temp);

        let mut gpu_load_history = Vec::with_capacity(120);
        gpu_load_history.push(initial_gpu_load);

        let mut mem_history = Vec::with_capacity(120);
        mem_history.push(snapshot.memory.percent);

        let mut app = Self {
            core,
            popup: None,
            active_tab: ActiveTab::Overview,
            expanded_details: false,
            unit: TemperatureUnit::Celsius,
            collector,
            snapshot,
            history,
            cpu_load_history,
            gpu_temp_history,
            gpu_load_history,
            mem_history,
            max_history: 120,
            config: TempTyleConfig::default(),
            expanded_cpu_cores: false,
        };

        // Try reading persisted config if available
        if let Ok(config_handler) = cosmic::cosmic_config::Config::new(APP_ID, crate::config::CONFIG_VERSION) {
            if let Ok(loaded) = TempTyleConfig::get_entry(&config_handler) {
                app.unit = if loaded.fahrenheit {
                    TemperatureUnit::Fahrenheit
                } else {
                    TemperatureUnit::Celsius
                };
                app.config = loaded;
            }
        }

        let initial_theme_task = match app.config.theme_pref {
            crate::config::ThemePreference::System => Task::none(),
            pref => cosmic::task::message(cosmic::Action::Cosmic(
                cosmic::app::Action::AppThemeChange(get_cosmic_theme(pref)),
            )),
        };

        (app, initial_theme_task)
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Tick | Message::Refresh => {
                self.snapshot = self.collector.collect();
                let temp = self.snapshot.summary.package_temp;
                self.history.push(temp);
                if self.history.len() > self.max_history {
                    self.history.remove(0);
                }

                self.cpu_load_history.push(self.snapshot.cpu.overall_usage);
                if self.cpu_load_history.len() > self.max_history {
                    self.cpu_load_history.remove(0);
                }

                if let Some(gpu) = &self.snapshot.gpu {
                    self.gpu_temp_history.push(gpu.edge_temp);
                    self.gpu_load_history.push(gpu.utilization_percent as f32);
                }
                if self.gpu_temp_history.len() > self.max_history {
                    self.gpu_temp_history.remove(0);
                }
                if self.gpu_load_history.len() > self.max_history {
                    self.gpu_load_history.remove(0);
                }

                self.mem_history.push(self.snapshot.memory.percent);
                if self.mem_history.len() > self.max_history {
                    self.mem_history.remove(0);
                }

                Task::none()
            }
            Message::SelectTab(tab) => {
                self.active_tab = tab;
                Task::none()
            }
            Message::ToggleExpandedDetails => {
                self.expanded_details = !self.expanded_details;
                Task::none()
            }
            Message::ToggleCpuCores => {
                self.expanded_cpu_cores = !self.expanded_cpu_cores;
                Task::none()
            }
            Message::SetTheme(pref) => {
                self.config.theme_pref = pref;
                if let Ok(config_handler) = cosmic::cosmic_config::Config::new(APP_ID, crate::config::CONFIG_VERSION) {
                    let _ = self.config.write_entry(&config_handler);
                }
                let theme = get_cosmic_theme(pref);
                cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::AppThemeChange(theme),
                ))
            }
            Message::SetUnit(unit) => {
                self.unit = unit;
                self.config.fahrenheit = unit == TemperatureUnit::Fahrenheit;
                if let Ok(config_handler) = cosmic::cosmic_config::Config::new(APP_ID, crate::config::CONFIG_VERSION) {
                    let _ = self.config.write_entry(&config_handler);
                }
                Task::none()
            }
            Message::SetInterval(sec) => {
                self.config.refresh_interval_secs = sec;
                if let Ok(config_handler) = cosmic::cosmic_config::Config::new(APP_ID, crate::config::CONFIG_VERSION) {
                    let _ = self.config.write_entry(&config_handler);
                }
                Task::none()
            }
            Message::ToggleUnits => {
                self.unit = match self.unit {
                    TemperatureUnit::Celsius => TemperatureUnit::Fahrenheit,
                    TemperatureUnit::Fahrenheit => TemperatureUnit::Celsius,
                };
                self.config.fahrenheit = self.unit == TemperatureUnit::Fahrenheit;
                // Save config asynchronously if handler exists
                if let Ok(config_handler) = cosmic::cosmic_config::Config::new(APP_ID, crate::config::CONFIG_VERSION) {
                    let _ = self.config.write_entry(&config_handler);
                }
                Task::none()
            }
            Message::ClosePopup => {
                if let Some(id) = self.popup.take() {
                    return cosmic::task::message(cosmic::Action::Cosmic(
                        cosmic::app::Action::Surface(destroy_popup(id)),
                    ));
                }
                Task::none()
            }
            Message::PopupClosed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
                Task::none()
            }
            Message::Surface(action) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(action),
                ));
            }
            Message::ConfigChanged(config) => {
                let theme_changed = config.theme_pref != self.config.theme_pref;
                self.unit = if config.fahrenheit {
                    TemperatureUnit::Fahrenheit
                } else {
                    TemperatureUnit::Celsius
                };
                self.config = config;
                if theme_changed {
                    let theme = get_cosmic_theme(self.config.theme_pref);
                    return cosmic::task::message(cosmic::Action::Cosmic(
                        cosmic::app::Action::AppThemeChange(theme),
                    ));
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let have_popup = self.popup;
        let suggested_size = self.core.applet.suggested_size(true);
        let sp = cosmic::theme::spacing();

        let status = self.snapshot.summary.thermal_status;
        let [r, g, b] = status.rgb();
        let hex_color = format!(
            "#{:02x}{:02x}{:02x}",
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8
        );
        let icon = widget::icon::icon(widget::icon::from_svg_bytes(
            crate::views::hero::thermometer_svg(&hex_color).into_bytes(),
        ))
        .size(suggested_size.0);

        let temp_label = self.unit.format_temp_short(self.snapshot.summary.package_temp);
        let temp_widget = widget::text::body(temp_label)
            .size(13);

        let content: Element<'_, Self::Message> = if self.core.applet.is_horizontal() {
            cosmic::iced::widget::row![icon, temp_widget]
                .spacing(sp.space_xxxs)
                .align_y(Alignment::Center)
                .into()
        } else {
            cosmic::iced::widget::column![icon, temp_widget]
                .spacing(2)
                .align_x(Alignment::Center)
                .into()
        };

        let horizontal = self.core.applet.is_horizontal();
        let pad = self.core.applet.suggested_padding(true).0;

        let button = cosmic::widget::button::custom(content)
            .padding(if horizontal { [0, pad] } else { [pad, 0] })
            .class(cosmic::theme::Button::AppletIcon)
            .on_press_with_rectangle(move |offset, bounds| {
                if have_popup.is_some() {
                    Message::ClosePopup
                } else {
                    Message::Surface(app_popup::<AppModel>(
                        |_| cosmic::surface::action::LiveSettings {
                            blur: Some(true),
                            ..Default::default()
                        },
                        move |state: &mut AppModel| {
                            let new_id = Id::unique();
                            state.popup = Some(new_id);
                            let initial_width = views::popup_width(state.active_tab);
                            let mut popup_settings = state.core.applet.get_popup_settings(
                                state.core.main_window_id().unwrap(),
                                new_id,
                                Some((initial_width as u32, 600)),
                                None,
                                None,
                            );
                            popup_settings.positioner.size_limits = cosmic::iced::Limits::NONE
                                .min_height(1.0)
                                .min_width(400.0)
                                .max_width(780.0)
                                .max_height(1400.0);
                            popup_settings.positioner.anchor_rect = Rectangle {
                                x: (bounds.x - offset.x) as i32,
                                y: (bounds.y - offset.y) as i32,
                                width: bounds.width as i32,
                                height: bounds.height as i32,
                            };
                            let (icon_w, icon_h) = state.core.applet.suggested_size(true);
                            let (_, minor) = state.core.applet.suggested_padding(true);
                            let (slot, extent) = if state.core.applet.is_horizontal() {
                                (f32::from(icon_h + 2 * minor), bounds.height)
                            } else {
                                (f32::from(icon_w + 2 * minor), bounds.width)
                            };
                            let shortfall = ((slot - extent) / 2.0).max(0.0).round() as i32;
                            let (ox, oy) = popup_settings.positioner.offset;
                            popup_settings.positioner.offset =
                                (ox + ox.signum() * shortfall, oy + oy.signum() * shortfall);
                            popup_settings
                        },
                        None,
                    ))
                }
            });

        widget::autosize::autosize(button, AUTOSIZE_MAIN_ID.clone()).into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let width = views::popup_width(self.active_tab);
        let content = views::view_popup(
            &self.snapshot,
            &self.history,
            &self.cpu_load_history,
            &self.gpu_temp_history,
            &self.gpu_load_history,
            &self.mem_history,
            self.active_tab,
            self.expanded_details,
            self.expanded_cpu_cores,
            self.unit,
            self.is_dark(),
            &self.config,
        );
        self.popup_container(content, width)
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let interval = self.config.refresh_interval_secs.max(1);
        let timer = cosmic::iced::time::every(Duration::from_secs(interval)).map(|_| Message::Tick);

        let config_sub = self
            .core()
            .watch_config::<TempTyleConfig>(APP_ID)
            .map(|update| Message::ConfigChanged(update.config));

        Subscription::batch(vec![timer, config_sub])
    }

    fn on_close_requested(&self, id: Id) -> Option<Self::Message> {
        Some(Message::PopupClosed(id))
    }
}
