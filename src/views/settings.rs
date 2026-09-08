//! Settings tab: theme, temperature unit and refresh rate.

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, column, row, text};
use cosmic::Element;

use crate::app::Message;
use crate::config::{TempTyleConfig, ThemePreference};
use crate::hardware::types::TemperatureUnit;
use crate::views::nav_row;

/// Refresh rates offered, as `(seconds, label)`.
const INTERVALS: [(u64, &str); 4] =
    [(1, "1s (Fast)"), (2, "2s (Normal)"), (3, "3s"), (5, "5s (Battery-friendly)")];

pub fn view<'a>(config: &TempTyleConfig, unit: TemperatureUnit) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();

    column![
        nav_row("Settings"),
        theme_section(config.theme_pref, sp.space_xs),
        section(
            "Temperature Unit",
            row![
                choice("Celsius (°C)", unit == TemperatureUnit::Celsius)
                    .on_press(Message::SetUnit(TemperatureUnit::Celsius)),
                choice("Fahrenheit (°F)", unit == TemperatureUnit::Fahrenheit)
                    .on_press(Message::SetUnit(TemperatureUnit::Fahrenheit)),
            ]
            .spacing(sp.space_s),
            sp.space_xs,
        ),
        section(
            "Refresh Interval",
            INTERVALS.iter().fold(row![].spacing(sp.space_xs), |acc, &(secs, label)| {
                acc.push(
                    choice(label, config.refresh_interval_secs == secs)
                        .on_press(Message::SetInterval(secs)),
                )
            }),
            sp.space_xs,
        ),
    ]
    .spacing(sp.space_m)
    .width(Length::Fill)
    .into()
}

fn theme_section<'a>(current: ThemePreference, spacing: u16) -> Element<'a, Message> {
    let option = |pref: ThemePreference, label: &'static str, desc: &'static str| {
        let selected = current == pref;
        let content = column![
            row![text(if selected { "● " } else { "○ " }).size(12), text(label).size(13)]
                .align_y(Alignment::Center)
                .spacing(4),
            text::caption(desc).size(10),
        ]
        .spacing(2);

        button::custom(content)
            .padding([8, 12])
            .width(Length::Fill)
            .class(button_class(selected))
            .on_press(Message::SetTheme(pref))
    };

    let options = column![
        option(
            ThemePreference::System,
            "System / Adapt to Desktop Theme",
            "Automatically switches between dark and light to match COSMIC"
        ),
        option(
            ThemePreference::Dark,
            "Black (Always Dark)",
            "Deep dark background with high-contrast emerald and cyan accents"
        ),
        option(
            ThemePreference::Light,
            "White (Always Light)",
            "Clean light background with crisp dark typography"
        ),
    ]
    .spacing(spacing)
    .width(Length::Fill);

    column![
        text::title3("Theme Adaptation").size(14),
        text::caption(
            "Choose whether Hardware Monitor follows your desktop theme or stays locked \
             to a light or dark appearance."
        )
        .size(11),
        options,
    ]
    .spacing(spacing)
    .width(Length::Fill)
    .into()
}

/// A titled block of settings.
fn section<'a>(
    title: &'a str,
    content: impl Into<Element<'a, Message>>,
    spacing: u16,
) -> Element<'a, Message> {
    column![text::title3(title).size(14), content.into()]
        .spacing(spacing)
        .width(Length::Fill)
        .into()
}

/// A toggle button that reads as selected or not.
fn choice(label: &str, selected: bool) -> cosmic::widget::Button<'_, Message> {
    button::custom(text(label).size(12)).padding([6, 16]).class(button_class(selected))
}

fn button_class(selected: bool) -> cosmic::theme::Button {
    if selected {
        cosmic::theme::Button::Suggested
    } else {
        cosmic::theme::Button::Standard
    }
}
