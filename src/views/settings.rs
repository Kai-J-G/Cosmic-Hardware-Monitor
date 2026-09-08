use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, column, row, text};
use cosmic::Element;

use crate::app::{ActiveTab, Message};
use crate::config::ThemePreference;
use crate::hardware::types::TemperatureUnit;

pub fn view_settings<'a>(
    current_theme: ThemePreference,
    current_unit: TemperatureUnit,
    current_interval: u64,
    _is_dark: bool,
) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();

    // 1. Header with back button
    let back_btn = button::text("← Overview")
        .padding([6, 12])
        .class(cosmic::theme::Button::Standard)
        .on_press(Message::SelectTab(ActiveTab::Overview));

    let title = text::title3("Settings").size(15);
    let nav_row = row![back_btn, cosmic::iced::widget::Space::new().width(Length::Fill), title]
        .align_y(Alignment::Center)
        .width(Length::Fill);

    // 2. Theme Adaptation Section
    let theme_title = text::title3("Theme Adaptation (Desktop Appearance)").size(14);
    let theme_desc = text::caption(
        "Select whether Hardware Monitor automatically adapts to your desktop theme or stays locked to black or white."
    )
    .size(11);

    let theme_btn = |pref: ThemePreference, label: &'static str, desc: &'static str| {
        let is_selected = current_theme == pref;
        let btn_content = column![
            row![
                text(if is_selected { "● " } else { "○ " }).size(12),
                text(label).size(13)
            ]
            .align_y(Alignment::Center)
            .spacing(4),
            text::caption(desc).size(10)
        ]
        .spacing(2);

        button::custom(btn_content)
            .padding([8, 12])
            .width(Length::Fill)
            .class(if is_selected {
                cosmic::theme::Button::Suggested
            } else {
                cosmic::theme::Button::Standard
            })
            .on_press(Message::SetTheme(pref))
    };

    let theme_options = column![
        theme_btn(
            ThemePreference::System,
            "System / Adapt to Desktop Theme",
            "Automatically switches between dark and light to match COSMIC"
        ),
        theme_btn(
            ThemePreference::Dark,
            "Black (Always Dark)",
            "Deep dark background with high-contrast emerald & cyan accents"
        ),
        theme_btn(
            ThemePreference::Light,
            "White (Always Light)",
            "Clean light background with crisp dark typography"
        ),
    ]
    .spacing(sp.space_xs)
    .width(Length::Fill);

    let theme_box = column![theme_title, theme_desc, theme_options]
        .spacing(sp.space_xs)
        .width(Length::Fill);

    // 3. Temperature Unit Section
    let unit_title = text::title3("Temperature Unit").size(14);
    let unit_btn = |unit: TemperatureUnit, label: &'static str| {
        let is_selected = current_unit == unit;
        button::custom(text(label).size(12))
            .padding([6, 16])
            .class(if is_selected {
                cosmic::theme::Button::Suggested
            } else {
                cosmic::theme::Button::Standard
            })
            .on_press(Message::SetUnit(unit))
    };

    let unit_row = row![
        unit_btn(TemperatureUnit::Celsius, "Celsius (°C)"),
        unit_btn(TemperatureUnit::Fahrenheit, "Fahrenheit (°F)"),
    ]
    .spacing(sp.space_s);

    let unit_box = column![unit_title, unit_row]
        .spacing(sp.space_xs)
        .width(Length::Fill);

    // 4. Refresh Interval Section
    let interval_title = text::title3("Refresh Interval").size(14);
    let interval_btn = |sec: u64, label: &'static str| {
        let is_selected = current_interval == sec;
        button::custom(text(label).size(11))
            .padding([4, 10])
            .class(if is_selected {
                cosmic::theme::Button::Suggested
            } else {
                cosmic::theme::Button::Standard
            })
            .on_press(Message::SetInterval(sec))
    };

    let interval_row = row![
        interval_btn(1, "1s (Fast)"),
        interval_btn(2, "2s (Normal)"),
        interval_btn(3, "3s"),
        interval_btn(5, "5s (Battery-friendly)"),
    ]
    .spacing(sp.space_xs);

    let interval_box = column![interval_title, interval_row]
        .spacing(sp.space_xs)
        .width(Length::Fill);

    column![nav_row, theme_box, unit_box, interval_box]
        .spacing(sp.space_m)
        .width(Length::Fill)
        .into()
}
