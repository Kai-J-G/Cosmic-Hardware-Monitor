//! Settings tab: theme, temperature unit and refresh rate.

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, column, row, text};
use cosmic::Element;

use crate::app::Message;
use crate::fl;
use crate::config::{HardwareMonitorConfig, ThemePreference};
use crate::hardware::types::TemperatureUnit;
use crate::views::nav_row;

/// Refresh rates offered, paired with the id of their button label.
///
/// Every reading is taken fresh on each tick, so a faster interval means more
/// file reads — hence the hints in the labels.
const INTERVALS: [(u64, fn() -> String); 4] = [
    (1, || fl!("interval-1s")),
    (2, || fl!("interval-2s")),
    (3, || fl!("interval-3s")),
    (5, || fl!("interval-5s")),
];

/// Theme choices, each with a name and a line explaining it.
fn themes() -> [(ThemePreference, String, String); 3] {
    [
        (ThemePreference::System, fl!("theme-system"), fl!("theme-system-description")),
        (ThemePreference::Dark, fl!("theme-dark"), fl!("theme-dark-description")),
        (ThemePreference::Light, fl!("theme-light"), fl!("theme-light-description")),
    ]
}

pub fn view<'a>(config: &HardwareMonitorConfig, unit: TemperatureUnit) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();

    column![
        nav_row(fl!("settings")),
        theme_section(config.theme_pref),
        unit_section(unit),
        interval_section(config.refresh_interval_secs),
    ]
    .spacing(sp.space_m)
    .width(Length::Fill)
    .into()
}

/// Radio-style list of theme choices, each with a line explaining it.
fn theme_section<'a>(current: ThemePreference) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();
    let mut options = column![].spacing(sp.space_xs).width(Length::Fill);

    for (preference, name, description) in themes() {
        let selected = current == preference;
        let label = column![
            row![
                // A filled bullet marks the active choice.
                text(if selected { "● " } else { "○ " }).size(12),
                text(name).size(13),
            ]
            .align_y(Alignment::Center)
            .spacing(4),
            text::caption(description).size(10),
        ]
        .spacing(2);

        options = options.push(
            button::custom(label)
                .padding([8, 12])
                .width(Length::Fill)
                .class(button_class(selected))
                .on_press(Message::SetTheme(preference)),
        );
    }

    section(fl!("theme-title"), Some(fl!("theme-description")), options)
}

fn unit_section<'a>(current: TemperatureUnit) -> Element<'a, Message> {
    let choices = row![
        choice(fl!("unit-celsius"), current == TemperatureUnit::Celsius)
            .on_press(Message::SetUnit(TemperatureUnit::Celsius)),
        choice(fl!("unit-fahrenheit"), current == TemperatureUnit::Fahrenheit)
            .on_press(Message::SetUnit(TemperatureUnit::Fahrenheit)),
    ]
    .spacing(cosmic::theme::spacing().space_s);

    section(fl!("unit-title"), None, choices)
}

fn interval_section<'a>(current_secs: u64) -> Element<'a, Message> {
    let mut choices = row![].spacing(cosmic::theme::spacing().space_xs);

    for (seconds, label) in INTERVALS {
        choices = choices
            .push(choice(label(), current_secs == seconds).on_press(Message::SetInterval(seconds)));
    }

    section(fl!("interval-title"), None, choices)
}

/// A titled block of settings, with an optional line of explanation.
fn section<'a>(
    title: String,
    description: Option<String>,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut block = column![text::title3(title).size(14)];

    if let Some(description) = description {
        block = block.push(text::caption(description).size(11));
    }

    block
        .push(content.into())
        .spacing(cosmic::theme::spacing().space_xs)
        .width(Length::Fill)
        .into()
}

/// A button that reads as selected or not.
fn choice<'a>(label: String, selected: bool) -> cosmic::widget::Button<'a, Message> {
    button::custom(text(label).size(12)).padding([6, 16]).class(button_class(selected))
}

/// Selected options use the accent colour; the rest are plain.
fn button_class(selected: bool) -> cosmic::theme::Button {
    if selected {
        cosmic::theme::Button::Suggested
    } else {
        cosmic::theme::Button::Standard
    }
}
