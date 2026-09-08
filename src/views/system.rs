use cosmic::iced::{Alignment, Color, Length};
use cosmic::widget::{button, column, container, determinate_linear, row, text};
use cosmic::Element;

use crate::app::{ActiveTab, Message};
use crate::hardware::types::{SystemInfo, TemperatureUnit};

#[allow(dead_code)]
pub fn view_system<'a>(system: &'a SystemInfo, unit: TemperatureUnit) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();

    // 0. Breadcrumb Navigation
    let back_btn = button::text("← Overview")
        .padding([6, 12])
        .class(cosmic::theme::Button::Standard)
        .on_press(Message::SelectTab(ActiveTab::Overview));

    let title = text::title3("System Telemetry & RAM").size(14);
    let nav_row = row![back_btn, cosmic::iced::widget::Space::new().width(Length::Fill), title]
        .align_y(Alignment::Center)
        .width(Length::Fill);

    // 1. RAM Memory Bar & SPD Sensors
    let ram_used_gb = system.mem_used_bytes as f64 / 1_073_741_824.0;
    let ram_total_gb = system.mem_total_bytes as f64 / 1_073_741_824.0;
    let ram_title = text::body(format!(
        "Memory (RAM): {:.2} GB / {:.2} GB ({:.1}%)",
        ram_used_gb, ram_total_gb, system.mem_percent
    ))
    .size(13);

    let ram_bar = determinate_linear((system.mem_percent / 100.0).clamp(0.0, 1.0));

    // DDR5 SPD temperatures if present
    let mut spd_chips = row![].spacing(sp.space_xs).align_y(Alignment::Center);
    for (name, temp) in &system.spd_temps {
        let txt = text::caption(format!("{name}: {}", unit.format_temp(*temp))).size(11);
        let badge = container(txt)
            .padding([2, 6])
            .class(cosmic::theme::Container::Custom(Box::new(|_| {
                cosmic::iced::widget::container::Style {
                    background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.08).into()),
                    border: cosmic::iced::border::rounded(6.0),
                    ..Default::default()
                }
            })));
        spd_chips = spd_chips.push(badge);
    }

    let ram_card = container(
        column![ram_title, ram_bar, spd_chips]
            .spacing(sp.space_xs)
            .width(Length::Fill),
    )
    .padding(sp.space_m)
    .class(cosmic::theme::Container::Custom(Box::new(|_theme| {
        cosmic::iced::widget::container::Style {
            background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.03).into()),
            border: cosmic::iced::Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
                width: 1.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        }
    })))
    .width(Length::Fill);

    // 2. System Details Card
    let hours = system.uptime_seconds / 3600;
    let mins = (system.uptime_seconds % 3600) / 60;
    let uptime_str = if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    };

    let spec_item = |title: &'static str, val: String| {
        column![text::caption(title).size(10), text::body(val).size(12)].spacing(1)
    };

    let col1 = column![
        spec_item("Operating System", system.os_name.clone()),
        spec_item("Kernel", system.kernel.clone())
    ]
    .spacing(sp.space_xs)
    .width(Length::FillPortion(1));

    let col2 = column![
        spec_item("Host", system.hostname.clone()),
        spec_item("Uptime", uptime_str)
    ]
    .spacing(sp.space_xs)
    .width(Length::FillPortion(1));

    let load_str = format!(
        "{:.2}, {:.2}, {:.2}",
        system.load_avg[0], system.load_avg[1], system.load_avg[2]
    );
    let col3 = column![
        spec_item("Load Avg", load_str),
        spec_item("Sensors Engine", "Linux Sysfs Direct".to_string())
    ]
    .spacing(sp.space_xs)
    .width(Length::FillPortion(1));

    let specs_row = row![col1, col2, col3]
        .spacing(sp.space_s)
        .width(Length::Fill);

    let system_card = container(specs_row)
        .padding(sp.space_m)
        .class(cosmic::theme::Container::Custom(Box::new(|_theme| {
            cosmic::iced::widget::container::Style {
                background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.03).into()),
                border: cosmic::iced::Border {
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
                    width: 1.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            }
        })))
        .width(Length::Fill);

    column![nav_row, ram_card, system_card]
        .spacing(sp.space_m)
        .width(Length::Fill)
        .into()
}
