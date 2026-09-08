use cosmic::iced::{Alignment, Color, Length};
use cosmic::widget::{button, column, container, icon, row, text};
use cosmic::Element;

use crate::app::Message;
use crate::hardware::types::{HardwareSnapshot, TemperatureUnit};
use crate::sparkline::Sparkline;

pub fn thermometer_svg(color_hex: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="34" height="34" viewBox="0 0 16 16" fill="{color_hex}">
  <path d="M10 5a2 2 0 1 0-4 0v4.2a3 3 0 1 0 4 0V5zm-2-1a1 1 0 0 1 1 1v4.5a.5.5 0 0 0 .2.4 2 2 0 1 1-2.4 0 .5.5 0 0 0 .2-.4V5a1 1 0 0 1 1-1zm0 7a1 1 0 1 0 0 2 1 1 0 0 0 0-2z"/>
</svg>"#
    )
}

#[allow(dead_code)]
pub fn view_hero<'a>(
    snapshot: &'a HardwareSnapshot,
    history: &'a [f32],
    unit: TemperatureUnit,
) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();
    let status = snapshot.summary.thermal_status;
    let [r, g, b] = status.rgb();
    let state_color = Color::from_rgb(r, g, b);

    // 1. Thermometer icon box with subtle tinted glow and rounded border
    let hex_color = format!(
        "#{:02x}{:02x}{:02x}",
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8
    );
    let therm_icon = icon::icon(icon::from_svg_bytes(thermometer_svg(&hex_color).into_bytes()))
        .size(34);

    let icon_box = container(therm_icon)
        .padding(14)
        .class(cosmic::theme::Container::Custom(Box::new(move |_theme| {
            cosmic::iced::widget::container::Style {
                background: Some(Color::from_rgba(r, g, b, 0.14).into()),
                border: cosmic::iced::Border {
                    color: Color::from_rgba(r, g, b, 0.35),
                    width: 1.0,
                    radius: 16.0.into(),
                },
                icon_color: Some(Color::from_rgb(r, g, b)),
                ..Default::default()
            }
        })));

    // 2. State label and large temperature reading
    let status_label = text::caption(status.label()).size(13);

    let (temp_val, temp_unit) = unit.format_temp_val(snapshot.summary.package_temp);

    let temp_number = text::title1(temp_val).size(48);
    let temp_unit_txt = text::title3(temp_unit).size(22);

    let temp_row = button::custom(
        row![temp_number, temp_unit_txt]
            .spacing(4)
            .align_y(Alignment::Start),
    )
    .class(cosmic::widget::button::ButtonClass::Link)
    .on_press(Message::ToggleUnits);

    let temp_block = column![status_label, temp_row]
        .spacing(sp.space_xxxs)
        .align_x(Alignment::Start);

    let left_hero = row![icon_box, temp_block]
        .spacing(sp.space_m)
        .align_y(Alignment::Center)
        .width(Length::Shrink);

    // 3. Sparkline history chart
    let sparkline = Sparkline::new(history, state_color).view(Length::Fill, Length::Fixed(72.0));

    let history_label = text::caption(if history.len() < 2 {
        "Gathering data…"
    } else {
        "Thermal History (10 min)"
    })
    .size(10);

    let history_col = column![sparkline, history_label]
        .spacing(4)
        .align_x(Alignment::End)
        .width(Length::Fill);

    // Top row: Left (Icon + Temp) and Right (Sparkline)
    row![left_hero, history_col]
        .spacing(sp.space_l)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .into()
}
