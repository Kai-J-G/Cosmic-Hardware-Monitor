use cosmic::iced::{Alignment, Color, Length};
use cosmic::widget::{column, container, determinate_linear, icon, mouse_area, row, text};
use cosmic::Element;

use crate::app::{ActiveTab, Message};
use crate::hardware::types::{HardwareSnapshot, TemperatureUnit, ThermalStatus};
use crate::views::circular_gauge::CircularGauge;

pub fn view_overview<'a>(
    snapshot: &'a HardwareSnapshot,
    expanded_details: bool,
    unit: TemperatureUnit,
    is_dark: bool,
) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();
    let emerald = Color::from_rgb(0.0, 0.90, 0.46);

    // -------------------------------------------------------------
    // 1. First View: 4 Circular Dials: [ CPU ] [ GPU ] [ Memory ] [ Disk ]
    // -------------------------------------------------------------
    let cpu_pct = snapshot.cpu.overall_usage;
    let cpu_temp_str = unit.format_temp_short(snapshot.cpu.package_temp);
    let cpu_gauge = CircularGauge::new(
        cpu_pct,
        format!("{:.0}%", cpu_pct),
        "CPU",
        emerald,
    )
    .with_sub_text(Some(cpu_temp_str))
    .with_theme(is_dark);

    let (gpu_pct, gpu_temp_sub) = if let Some(gpu) = &snapshot.gpu {
        (
            gpu.utilization_percent as f32,
            Some(unit.format_temp_short(gpu.edge_temp)),
        )
    } else {
        (0.0, None)
    };
    let gpu_gauge = CircularGauge::new(
        gpu_pct,
        format!("{:.0}%", gpu_pct),
        "GPU",
        emerald,
    )
    .with_sub_text(gpu_temp_sub)
    .with_theme(is_dark);

    let mem_pct = snapshot.memory.percent;
    let mem_gauge = CircularGauge::new(
        mem_pct,
        format!("{:.0}%", mem_pct),
        "Memory",
        emerald,
    )
    .with_theme(is_dark);

    let disk_pct = snapshot.system.disk_percent;
    let disk_gauge = CircularGauge::new(
        disk_pct,
        format!("{:.0}%", disk_pct),
        "Disk",
        emerald,
    )
    .with_theme(is_dark);

    let dial_size = 66.0;
    let dials_row = row![
        mouse_area(cpu_gauge.view(dial_size))
            .on_press(Message::SelectTab(ActiveTab::CpuDetail))
            .interaction(cosmic::iced::mouse::Interaction::Pointer),
        mouse_area(gpu_gauge.view(dial_size))
            .on_press(Message::SelectTab(ActiveTab::GpuDetail))
            .interaction(cosmic::iced::mouse::Interaction::Pointer),
        mouse_area(mem_gauge.view(dial_size))
            .on_press(Message::SelectTab(ActiveTab::MemoryDetail))
            .interaction(cosmic::iced::mouse::Interaction::Pointer),
        mouse_area(disk_gauge.view(dial_size))
            .on_press(Message::SelectTab(ActiveTab::StorageDetail))
            .interaction(cosmic::iced::mouse::Interaction::Pointer),
    ]
    .spacing(18)
    .align_y(Alignment::Center);

    let dials_container = container(dials_row)
        .align_x(cosmic::iced::alignment::Horizontal::Center)
        .width(Length::Fill);

    // Click-down toggle button / arrow
    let expand_arrow_text = if expanded_details { "⌃" } else { "⌄" };
    let expand_btn = mouse_area(
        container(
            row![
                text(expand_arrow_text).size(15)
            ]
            .align_y(Alignment::Center)
        )
        .padding([2, 20])
        .class(cosmic::theme::Container::Custom(Box::new(move |_theme| {
            cosmic::iced::widget::container::Style {
                background: Some(if is_dark {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.05).into()
                } else {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.04).into()
                }),
                border: cosmic::iced::Border {
                    color: if is_dark {
                        Color::from_rgba(1.0, 1.0, 1.0, 0.08)
                    } else {
                        Color::from_rgba(0.0, 0.0, 0.0, 0.08)
                    },
                    width: 1.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            }
        })))
    )
    .on_press(Message::ToggleExpandedDetails)
    .interaction(cosmic::iced::mouse::Interaction::Pointer);

    let expand_btn_container = container(expand_btn)
        .align_x(cosmic::iced::alignment::Horizontal::Center)
        .width(Length::Fill);

    // If collapsed: show clear Network title & speeds underneath the arrow
    if !expanded_details {
        let net_title = text::caption("Network").size(11);
        let net_row = row![
            text(format!("↑ {}", format_net_rate(snapshot.system.net_tx_kbs))).size(11),
            green_dot(),
            text(format!("{} ↓", format_net_rate(snapshot.system.net_rx_kbs))).size(11),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        let net_box = column![net_title, net_row]
            .spacing(2)
            .align_x(Alignment::Center);

        let net_container = container(net_box)
            .align_x(cosmic::iced::alignment::Horizontal::Center)
            .width(Length::Fill);

        return column![dials_container, expand_btn_container, net_container]
            .spacing(sp.space_s)
            .width(Length::Fill)
            .into();
    }

    // -------------------------------------------------------------
    // 2. Expanded Details Section
    // -------------------------------------------------------------
    let mut details_col = column![].spacing(sp.space_s).width(Length::Fill);

    details_col = details_col.push(divider(is_dark));

    // --- Section 1: Network Speed ---
    let net_title = container(
        row![
            icon::from_name("network-transmit-receive-symbolic").size(14),
            text::title3("Network").size(13)
        ]
        .spacing(6)
        .align_y(Alignment::Center)
    )
    .align_x(cosmic::iced::alignment::Horizontal::Center)
    .width(Length::Fill);

    let net_rates = row![
        text(format!("Upload: ↑ {}", format_net_rate(snapshot.system.net_tx_kbs))).size(12),
        green_dot(),
        text(format!("Download: {} ↓", format_net_rate(snapshot.system.net_rx_kbs))).size(12),
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    let net_rates_container = container(net_rates)
        .align_x(cosmic::iced::alignment::Horizontal::Center)
        .width(Length::Fill);

    let net_section = column![net_title, net_rates_container]
        .spacing(4)
        .width(Length::Fill);

    details_col = details_col.push(net_section);
    details_col = details_col.push(divider(is_dark));

    // --- Section 2: CPU Usage & Load avg side-by-side ---
    let cpu_usage_col = column![
        text::title3("CPU Usage").size(13),
        row![
            color_box(Color::from_rgb(0.23, 0.51, 0.96)),
            text("user").size(12),
            cosmic::iced::widget::Space::new().width(Length::Fill),
            text(format!("{:.0}%", snapshot.cpu.user_usage)).size(12)
        ]
        .spacing(6)
        .align_y(Alignment::Center),
        row![
            color_box(Color::from_rgb(0.94, 0.27, 0.27)),
            text("sys").size(12),
            cosmic::iced::widget::Space::new().width(Length::Fill),
            text(format!("{:.0}%", snapshot.cpu.sys_usage)).size(12)
        ]
        .spacing(6)
        .align_y(Alignment::Center),
        row![
            color_box(if is_dark {
                Color::from_rgba(1.0, 1.0, 1.0, 0.35)
            } else {
                Color::from_rgba(0.0, 0.0, 0.0, 0.35)
            }),
            text("idle").size(12),
            cosmic::iced::widget::Space::new().width(Length::Fill),
            text(format!("{:.0}%", snapshot.cpu.idle_usage)).size(12)
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    ]
    .spacing(4)
    .width(Length::FillPortion(1));

    let load_avg_col = column![
        text::title3("Load avg").size(13),
        row![
            text("1m").size(12),
            cosmic::iced::widget::Space::new().width(Length::Fill),
            text(format!("{:.2}", snapshot.system.load_avg[0])).size(12)
        ]
        .align_y(Alignment::Center),
        row![
            text("5m").size(12),
            cosmic::iced::widget::Space::new().width(Length::Fill),
            text(format!("{:.2}", snapshot.system.load_avg[1])).size(12)
        ]
        .align_y(Alignment::Center),
        row![
            text("15m").size(12),
            cosmic::iced::widget::Space::new().width(Length::Fill),
            text(format!("{:.2}", snapshot.system.load_avg[2])).size(12)
        ]
        .align_y(Alignment::Center),
    ]
    .spacing(4)
    .width(Length::FillPortion(1));

    let usage_and_load = row![cpu_usage_col, load_avg_col]
        .spacing(sp.space_l)
        .width(Length::Fill);

    details_col = details_col.push(usage_and_load);
    details_col = details_col.push(divider(is_dark));

    // --- Section 3: GPU Usage ---
    let gpu_header = text::title3("GPU Usage").size(13);
    let gpu_content: Element<'a, Message> = if let Some(gpu) = &snapshot.gpu {
        let mut left_lines = column![
            row![
                text::caption("Memory Used").size(11),
                cosmic::iced::widget::Space::new().width(Length::Fill),
                text(format_bytes(gpu.vram_used_bytes)).size(11)
            ]
            .align_y(Alignment::Center),
            row![
                text::caption("Memory Size").size(11),
                cosmic::iced::widget::Space::new().width(Length::Fill),
                text(format_bytes(gpu.vram_total_bytes)).size(11)
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(3)
        .width(Length::FillPortion(1));

        if let Some(p) = gpu.power_draw_watts {
            left_lines = left_lines.push(
                row![
                    text::caption("Power Draw").size(11),
                    cosmic::iced::widget::Space::new().width(Length::Fill),
                    text(format!("{:.1} W", p)).size(11)
                ]
                .align_y(Alignment::Center),
            );
        }

        let mini_gauge = CircularGauge::new(
            gpu.utilization_percent as f32,
            format!("{}%", gpu.utilization_percent),
            "",
            emerald,
        )
        .with_theme(is_dark)
        .view(48.0);

        row![left_lines, mini_gauge]
            .spacing(sp.space_m)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .into()
    } else {
        text::caption("Integrated graphics or GPU idle")
            .size(11)
            .into()
    };

    let gpu_section = column![gpu_header, gpu_content]
        .spacing(6)
        .width(Length::Fill);

    details_col = details_col.push(gpu_section);
    details_col = details_col.push(divider(is_dark));

    // --- Section 4: Storage ---
    let used_gb = snapshot.system.disk_used_bytes as f64 / 1_073_741_824.0;
    let total_gb = snapshot.system.disk_total_bytes as f64 / 1_073_741_824.0;
    let storage_header = row![
        icon::from_name("drive-harddisk-symbolic").size(15),
        text::title3("Storage").size(13),
        cosmic::iced::widget::Space::new().width(Length::Fill),
        text::caption(format!("{:.1} GB / {:.1} GB ({:.0}%)", used_gb, total_gb, snapshot.system.disk_percent)).size(11),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let storage_bar = determinate_linear(snapshot.system.disk_percent / 100.0);

    let mut storage_drives_col = column![].spacing(3).width(Length::Fill);
    for drive in &snapshot.storage {
        let drive_row = row![
            text(&drive.name).size(12),
            cosmic::iced::widget::Space::new().width(Length::Fill),
            temp_badge(drive.composite_temp, unit),
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill);
        storage_drives_col = storage_drives_col.push(drive_row);
    }

    let storage_content = column![storage_header, storage_bar, storage_drives_col]
        .spacing(5)
        .width(Length::Fill);

    let storage_card = mouse_area(storage_content)
        .on_press(Message::SelectTab(ActiveTab::StorageDetail))
        .interaction(cosmic::iced::mouse::Interaction::Pointer);

    details_col = details_col.push(storage_card);
    details_col = details_col.push(divider(is_dark));

    // --- Section 5: Uptime ---
    let uptime_row = row![
        text("Uptime ›").size(12),
        cosmic::iced::widget::Space::new().width(Length::Fill),
        text(format_uptime(snapshot.system.uptime_seconds)).size(12),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill);

    details_col = details_col.push(uptime_row);

    column![dials_container, expand_btn_container, details_col]
        .spacing(sp.space_s)
        .width(Length::Fill)
        .into()
}

fn format_bytes(bytes: u64) -> String {
    let mb = bytes as f64 / (1024.0 * 1024.0);
    if mb >= 1024.0 {
        format!("{:.2} GB", mb / 1024.0)
    } else {
        format!("{:.0} MB", mb)
    }
}

fn format_net_rate(kbs: f32) -> String {
    if kbs >= 1024.0 {
        format!("{:.1} MB/s", kbs / 1024.0)
    } else {
        format!("{:.1} KB/s", kbs)
    }
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    if days > 0 {
        format!("{}d, {}h, {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{} hours, {} minutes", hours, minutes)
    } else {
        format!("{} minutes", minutes)
    }
}

fn temp_badge<'a, Message: 'static>(temp: f32, unit: TemperatureUnit) -> Element<'a, Message> {
    let status = ThermalStatus::from_celsius(temp);
    let [r, g, b] = status.rgb();
    container(
        text(unit.format_temp_short(temp)).size(11)
    )
    .padding([2, 8])
    .class(cosmic::theme::Container::Custom(Box::new(move |_| {
        cosmic::iced::widget::container::Style {
            background: Some(Color::from_rgba(r, g, b, 0.12).into()),
            border: cosmic::iced::Border {
                color: Color::from_rgba(r, g, b, 0.35),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }
    })))
    .into()
}

fn color_box<'a, Message: 'static>(color: Color) -> Element<'a, Message> {
    container(cosmic::iced::widget::Space::new())
        .width(Length::Fixed(8.0))
        .height(Length::Fixed(8.0))
        .class(cosmic::theme::Container::Custom(Box::new(move |_| {
            cosmic::iced::widget::container::Style {
                background: Some(color.into()),
                border: cosmic::iced::Border {
                    radius: 2.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })))
        .into()
}

fn green_dot<'a, Message: 'static>() -> Element<'a, Message> {
    container(cosmic::iced::widget::Space::new())
        .width(Length::Fixed(8.0))
        .height(Length::Fixed(8.0))
        .class(cosmic::theme::Container::Custom(Box::new(|_| {
            cosmic::iced::widget::container::Style {
                background: Some(Color::from_rgb(0.0, 0.90, 0.46).into()),
                border: cosmic::iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })))
        .into()
}

fn divider<'a, Message: 'static>(is_dark: bool) -> Element<'a, Message> {
    container(cosmic::iced::widget::Space::new())
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .class(cosmic::theme::Container::Custom(Box::new(move |_| {
            cosmic::iced::widget::container::Style {
                background: Some(if is_dark {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.07).into()
                } else {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.08).into()
                }),
                ..Default::default()
            }
        })))
        .into()
}
