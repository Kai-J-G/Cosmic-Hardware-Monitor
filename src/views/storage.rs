use cosmic::iced::{Alignment, Color, Length};
use cosmic::widget::{button, column, container, determinate_linear, icon, row, scrollable, text};
use cosmic::Element;

use crate::app::{ActiveTab, Message};
use crate::hardware::types::StorageMetrics;

pub fn view_storage<'a>(
    metrics: &'a StorageMetrics,
    is_dark: bool,
) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();
    let emerald = Color::from_rgb(0.0, 0.90, 0.46);
    let cyan = Color::from_rgb(0.0, 0.75, 1.0);

    // 1. Navigation Breadcrumb Header
    let back_btn = button::text("← Overview")
        .padding([6, 12])
        .class(cosmic::theme::Button::Standard)
        .on_press(Message::SelectTab(ActiveTab::Overview));

    let title = text::title3("Storage & Partition Metrics").size(14);
    let nav_row = row![back_btn, cosmic::iced::widget::Space::new().width(Length::Fill), title]
        .align_y(Alignment::Center)
        .width(Length::Fill);

    // 2. Read / Write Speed Cards
    let speed_card = |label: &'static str, speed_str: String, icon_name: &'static str, accent: Color| {
        let ico = icon::from_name(icon_name).size(18);
        let lbl = text::caption(label).size(11);
        let val = text::title3(speed_str).size(17);

        let content = row![
            ico,
            column![lbl, val].spacing(2),
        ]
        .spacing(sp.space_s)
        .align_y(Alignment::Center);

        container(content)
            .padding([10, 14])
            .width(Length::FillPortion(1))
            .class(cosmic::theme::Container::Custom(Box::new(move |_theme| {
                cosmic::iced::widget::container::Style {
                    background: Some(if is_dark {
                        Color::from_rgba(accent.r, accent.g, accent.b, 0.08).into()
                    } else {
                        Color::from_rgba(accent.r, accent.g, accent.b, 0.06).into()
                    }),
                    border: cosmic::iced::Border {
                        color: Color::from_rgba(accent.r, accent.g, accent.b, if is_dark { 0.25 } else { 0.20 }),
                        width: 1.0,
                        radius: 12.0.into(),
                    },
                    ..Default::default()
                }
            })))
    };

    let speeds_row = row![
        speed_card("Read Speed", format!("↑ {}", format_rate(metrics.read_kbs)), "go-up-symbolic", cyan),
        speed_card("Write Speed", format!("↓ {}", format_rate(metrics.write_kbs)), "go-down-symbolic", emerald),
    ]
    .spacing(sp.space_s)
    .width(Length::Fill);

    // 3. Partitions with Free Space per Partition
    let partitions_title = text::title3("Mounted Partitions & Free Space").size(13);

    let mut parts_list = column![].spacing(sp.space_xs).width(Length::Fill);

    if metrics.partitions.is_empty() {
        parts_list = parts_list.push(text::caption("No mounted partitions detected.").size(12));
    } else {
        for part in &metrics.partitions {
            let part_icon = icon::from_name("drive-harddisk-symbolic").size(18);
            let mount_lbl = text::body(format!("Mount: {}", part.mount)).size(13);
            let fs_badge = container(text::caption(&part.filesystem).size(10))
                .padding([2, 6])
                .class(cosmic::theme::Container::Custom(Box::new(move |_theme| {
                    cosmic::iced::widget::container::Style {
                        background: Some(if is_dark {
                            Color::from_rgba(1.0, 1.0, 1.0, 0.06).into()
                        } else {
                            Color::from_rgba(0.0, 0.0, 0.0, 0.06).into()
                        }),
                        border: cosmic::iced::Border {
                            radius: 4.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                })));

            let top_line = row![
                part_icon,
                mount_lbl,
                fs_badge,
                cosmic::iced::widget::Space::new().width(Length::Fill),
                text::title3(format!("{:.1}% used", part.percent_used)).size(13),
            ]
            .spacing(sp.space_xs)
            .align_y(Alignment::Center)
            .width(Length::Fill);

            let bar = determinate_linear((part.percent_used / 100.0).clamp(0.0, 1.0));

            let free_str = format!("Free: {}", format_bytes(part.free_bytes));
            let used_total_str = format!("{} used of {}", format_bytes(part.used_bytes), format_bytes(part.total_bytes));

            let bottom_line = row![
                text::body(free_str).size(11),
                cosmic::iced::widget::Space::new().width(Length::Fill),
                text::caption(used_total_str).size(11),
            ]
            .align_y(Alignment::Center)
            .width(Length::Fill);

            let card_content = column![top_line, bar, bottom_line]
                .spacing(5)
                .width(Length::Fill);

            let part_card = container(card_content)
                .padding([sp.space_s, sp.space_m])
                .width(Length::Fill)
                .class(cosmic::theme::Container::Custom(Box::new(move |_theme| {
                    cosmic::iced::widget::container::Style {
                        background: Some(if is_dark {
                            Color::from_rgba(1.0, 1.0, 1.0, 0.03).into()
                        } else {
                            Color::from_rgba(0.0, 0.0, 0.0, 0.03).into()
                        }),
                        border: cosmic::iced::Border {
                            color: if is_dark {
                                Color::from_rgba(1.0, 1.0, 1.0, 0.06)
                            } else {
                                Color::from_rgba(0.0, 0.0, 0.0, 0.08)
                            },
                            width: 1.0,
                            radius: 12.0.into(),
                        },
                        ..Default::default()
                    }
                })));

            parts_list = parts_list.push(part_card);
        }
    }

    let scrollable_parts = scrollable(parts_list)
        .height(Length::Fixed(260.0))
        .width(Length::Fill);

    column![nav_row, speeds_row, partitions_title, scrollable_parts]
        .spacing(sp.space_m)
        .width(Length::Fill)
        .into()
}

fn format_rate(kbs: f32) -> String {
    if kbs >= 1024.0 {
        format!("{:.1} MB/s", kbs / 1024.0)
    } else {
        format!("{:.1} KB/s", kbs)
    }
}

fn format_bytes(bytes: u64) -> String {
    let mb = bytes as f64 / (1024.0 * 1024.0);
    if mb >= 1024.0 {
        format!("{:.2} GB", mb / 1024.0)
    } else {
        format!("{:.0} MB", mb)
    }
}
