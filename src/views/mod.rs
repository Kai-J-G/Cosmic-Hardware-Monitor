//! The popup UI: a header, then whichever tab is active.
//!
//! Views are pure functions of a [`HardwareSnapshot`] plus the small amount of
//! UI state the app holds; they own no state of their own.

pub mod circular_gauge;
pub mod cores;
pub mod fmt;
pub mod gpu;
pub mod memory;
pub mod overview;
pub mod panel;
pub mod settings;
pub mod sparkline;
pub mod storage;
pub mod style;
pub mod vertical_bar;

use cosmic::iced::{Alignment, Color, Length};
use cosmic::widget::{button, column, container, row, text, Space};
use cosmic::Element;

use crate::app::{ActiveTab, AppModel, Message};

/// Neutral accent for readings that carry no thermal meaning.
pub const EMERALD: Color = Color::from_rgb(0.0, 0.90, 0.46);

/// Each tab is laid out for its own content, so the popup resizes with it.
pub fn popup_width(active_tab: ActiveTab) -> f32 {
    match active_tab {
        ActiveTab::CpuDetail => 760.0,
        ActiveTab::GpuDetail => 740.0,
        ActiveTab::MemoryDetail | ActiveTab::StorageDetail => 640.0,
        ActiveTab::Settings => 480.0,
        ActiveTab::Overview => 440.0,
    }
}

/// Renders the popup: a shared header above the active tab's content.
pub fn view_popup(app: &AppModel) -> Element<'_, Message> {
    let sp = cosmic::theme::spacing();
    let snapshot = &app.snapshot;
    let (unit, is_dark) = (app.unit, app.is_dark());

    let content = match app.active_tab {
        ActiveTab::Overview => overview::view(snapshot, app.expanded_details, unit, is_dark),
        ActiveTab::CpuDetail => {
            cores::view(&snapshot.cpu, app.cpu_temp.as_slice(), unit, is_dark, app.expanded_cores)
        }
        ActiveTab::GpuDetail => {
            gpu::view(snapshot.gpu.as_ref(), app.gpu_temp.as_slice(), unit, is_dark)
        }
        ActiveTab::MemoryDetail => {
            memory::view(&snapshot.memory, app.memory_usage.as_slice(), is_dark)
        }
        ActiveTab::StorageDetail => storage::view(&snapshot.storage_metrics, is_dark),
        ActiveTab::Settings => settings::view(&app.config, unit),
    };

    column![header(app.active_tab), content]
        .spacing(sp.space_m)
        .padding([sp.space_s, sp.space_m, sp.space_m, sp.space_m])
        .width(Length::Fixed(popup_width(app.active_tab)))
        .into()
}

/// Centred title with a settings toggle pinned to the right.
fn header<'a>(active_tab: ActiveTab) -> Element<'a, Message> {
    let in_settings = active_tab == ActiveTab::Settings;
    let gutter = 40.0;

    let settings_btn = button::icon(cosmic::widget::icon::from_name("emblem-system-symbolic").size(16))
        .padding(8)
        .class(if in_settings { cosmic::theme::Button::Suggested } else { cosmic::theme::Button::Icon })
        // The gear doubles as a way back out of settings.
        .on_press(Message::SelectTab(if in_settings {
            ActiveTab::Overview
        } else {
            ActiveTab::Settings
        }));

    row![
        // Balances the button's width so the title sits truly centred.
        Space::new().width(Length::Fixed(gutter)),
        container(text::title3("Hardware Monitor").size(15))
            .align_x(Alignment::Center)
            .width(Length::Fill),
        container(settings_btn).align_x(Alignment::End).width(Length::Fixed(gutter)),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}

/// Breadcrumb shown at the top of every detail tab: a way back, then a title.
pub fn nav_row<'a>(title: impl Into<String>) -> Element<'a, Message> {
    row![
        button::text("← Overview")
            .padding([6, 12])
            .class(cosmic::theme::Button::Standard)
            .on_press(Message::SelectTab(ActiveTab::Overview)),
        Space::new().width(Length::Fill),
        text::title3(title.into()).size(14),
    ]
    .spacing(cosmic::theme::spacing().space_s)
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}

/// A headline reading: small label, large value, optional caption beneath.
///
/// `accent` tints the card, marking the reading as thermally significant.
pub fn stat_card<'a>(
    label: &'a str,
    value: String,
    sub: Option<String>,
    accent: Option<Color>,
    is_dark: bool,
) -> Element<'a, Message> {
    let mut content = column![text::caption(label).size(11), text::title1(value).size(24)]
        .spacing(2)
        .align_x(Alignment::Center);

    if let Some(sub) = sub {
        content = content.push(text::caption(sub).size(10));
    }

    container(content)
        .padding([10, 12])
        .width(Length::FillPortion(1))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .class(style::maybe_accent_card(accent, is_dark))
        .into()
}

/// A labelled sparkline in a card, used by every detail tab that plots history.
pub fn graph_card<'a>(
    label: &'a str,
    history: &'a [f32],
    color: Color,
    is_dark: bool,
) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();
    let chart = sparkline::Sparkline::new(history, color).view(Length::Fill, Length::Fixed(76.0));

    container(column![text::caption(label).size(11), chart].spacing(sp.space_xs).width(Length::Fill))
        .padding(sp.space_m)
        .width(Length::Fill)
        .class(style::card(is_dark))
        .into()
}

/// A label on the left and its value pushed to the right edge.
pub fn spread_row<'a>(
    label: impl Into<Element<'a, Message>>,
    value: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    row![label.into(), Space::new().width(Length::Fill), value.into()]
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .into()
}
