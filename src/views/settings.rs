//! Settings tab: theme, temperature unit and refresh rate.

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, column, row, text};
use cosmic::Element;

use crate::app::Message;
use crate::config::{TempTyleConfig, ThemePreference};
use crate::hardware::types::TemperatureUnit;
use crate::views::nav_row;

/// Refresh rates offered, as `(seconds, button label)`.
///
/// Every reading is taken fresh on each tick, so a faster interval means more
/// file reads — hence the hints in the labels.
const INTERVALS: [(u64, &str); 4] =
    [(1, "1s (Fast)"), (2, "2s (Normal)"), (3, "3s"), (5, "5s (Battery-friendly)")];

const THEMES: [(ThemePreference, &str, &str); 3] = [
    (
        ThemePreference::System,
        "System / Adapt to Desktop Theme",
        "Automatically switches between dark and light to match COSMIC",
    ),
    (
        ThemePreference::Dark,
        "Black (Always Dark)",
        "Deep dark background with high-contrast emerald and cyan accents",
    ),
    (
        ThemePreference::Light,
        "White (Always Light)",
        "Clean light background with crisp dark typography",
    ),
];

pub fn view<'a>(config: &TempTyleConfig, unit: TemperatureUnit) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();

    column![
        nav_row("Settings"),
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

    for (preference, name, description) in THEMES {
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

    section(
        "Theme Adaptation",
        Some(
            "Choose whether Hardware Monitor follows your desktop theme or stays locked \
             to a light or dark appearance.",
        ),
        options,
    )
}

fn unit_section<'a>(current: TemperatureUnit) -> Element<'a, Message> {
    let choices = row![
        choice("Celsius (°C)", current == TemperatureUnit::Celsius)
            .on_press(Message::SetUnit(TemperatureUnit::Celsius)),
        choice("Fahrenheit (°F)", current == TemperatureUnit::Fahrenheit)
            .on_press(Message::SetUnit(TemperatureUnit::Fahrenheit)),
    ]
    .spacing(cosmic::theme::spacing().space_s);

    section("Temperature Unit", None, choices)
}

fn interval_section<'a>(current_secs: u64) -> Element<'a, Message> {
    let mut choices = row![].spacing(cosmic::theme::spacing().space_xs);

    for (seconds, label) in INTERVALS {
        choices = choices
            .push(choice(label, current_secs == seconds).on_press(Message::SetInterval(seconds)));
    }

    section("Refresh Interval", None, choices)
}

/// A titled block of settings, with an optional line of explanation.
fn section<'a>(
    title: &'a str,
    description: Option<&'a str>,
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
fn choice(label: &str, selected: bool) -> cosmic::widget::Button<'_, Message> {
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
