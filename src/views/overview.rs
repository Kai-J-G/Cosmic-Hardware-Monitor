//! Overview tab: four dials, and an expandable panel of detail sections.

use cosmic::iced::{Alignment, Color, Length};
use cosmic::widget::{column, container, determinate_linear, icon, mouse_area, row, text};
use cosmic::Element;

use crate::app::{ActiveTab, Message};
use crate::hardware::types::{HardwareSnapshot, TemperatureUnit, ThermalStatus};
use crate::views::circular_gauge::CircularGauge;
use crate::views::{fmt, spread_row, style, EMERALD};

const DIAL_SIZE: f32 = 66.0;

pub fn view<'a>(
    snapshot: &'a HardwareSnapshot,
    expanded: bool,
    unit: TemperatureUnit,
    is_dark: bool,
) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();

    let body: Element<'a, Message> = if expanded {
        details(snapshot, unit, is_dark)
    } else {
        // Collapsed, the network rates are the one thing worth keeping visible.
        centered(
            column![
                text::caption("Network").size(11),
                net_rates(snapshot, 11, "↑ {}", "{} ↓"),
            ]
            .spacing(2)
            .align_x(Alignment::Center),
        )
    };

    column![centered(dials(snapshot, unit, is_dark)), centered(expand_toggle(expanded, is_dark)), body]
        .spacing(sp.space_s)
        .width(Length::Fill)
        .into()
}

/// The four headline dials, each a shortcut into its detail tab.
fn dials<'a>(
    snapshot: &'a HardwareSnapshot,
    unit: TemperatureUnit,
    is_dark: bool,
) -> Element<'a, Message> {
    let dial = |percent: f32, label: &'static str, sub: Option<String>, tab: ActiveTab| {
        let gauge = CircularGauge::new(percent, format!("{percent:.0}%"), label, EMERALD)
            .with_sub_text(sub)
            .with_theme(is_dark);

        mouse_area(gauge.view(DIAL_SIZE))
            .on_press(Message::SelectTab(tab))
            .interaction(cosmic::iced::mouse::Interaction::Pointer)
    };

    let (gpu_load, gpu_temp) = match &snapshot.gpu {
        Some(gpu) => (
            gpu.utilization_percent as f32,
            Some(unit.format_short(gpu.edge_temp)),
        ),
        None => (0.0, None),
    };
    let disk_pct = snapshot.storage_metrics.root().map_or(0.0, |root| root.percent_used);

    row![
        dial(
            snapshot.cpu.overall_usage,
            "CPU",
            Some(unit.format_short(snapshot.cpu.package_temp)),
            ActiveTab::CpuDetail,
        ),
        dial(gpu_load, "GPU", gpu_temp, ActiveTab::GpuDetail),
        dial(snapshot.memory.percent, "Memory", None, ActiveTab::MemoryDetail),
        dial(disk_pct, "Disk", None, ActiveTab::StorageDetail),
    ]
    .spacing(18)
    .align_y(Alignment::Center)
    .into()
}

/// The chevron that opens and closes the detail sections.
fn expand_toggle<'a>(expanded: bool, is_dark: bool) -> Element<'a, Message> {
    mouse_area(
        container(text(if expanded { "⌃" } else { "⌄" }).size(15))
            .padding([2, 20])
            .class(style::card(is_dark)),
    )
    .on_press(Message::ToggleExpandedDetails)
    .interaction(cosmic::iced::mouse::Interaction::Pointer)
    .into()
}

/// Everything revealed below the dials once expanded.
fn details<'a>(
    snapshot: &'a HardwareSnapshot,
    unit: TemperatureUnit,
    is_dark: bool,
) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();

    column![
        style::divider(is_dark),
        network_section(snapshot),
        style::divider(is_dark),
        row![cpu_usage_section(snapshot, is_dark), load_average_section(snapshot)]
            .spacing(sp.space_l)
            .width(Length::Fill),
        style::divider(is_dark),
        gpu_section(snapshot, is_dark),
        style::divider(is_dark),
        storage_section(snapshot, unit),
        style::divider(is_dark),
        spread_row(
            text("Uptime ›").size(12),
            text(fmt::uptime(snapshot.system.uptime_seconds)).size(12),
        ),
    ]
    .spacing(sp.space_s)
    .width(Length::Fill)
    .into()
}

fn network_section<'a>(snapshot: &'a HardwareSnapshot) -> Element<'a, Message> {
    column![
        centered(
            row![
                icon::from_name("network-transmit-receive-symbolic").size(14),
                text::title3("Network").size(13),
            ]
            .spacing(6)
            .align_y(Alignment::Center)
        ),
        centered(net_rates(snapshot, 12, "Upload: ↑ {}", "Download: {} ↓")),
    ]
    .spacing(4)
    .width(Length::Fill)
    .into()
}

/// Upload and download rates either side of a status dot.
fn net_rates<'a>(
    snapshot: &'a HardwareSnapshot,
    size: u16,
    up: &str,
    down: &str,
) -> Element<'a, Message> {
    let format = |template: &str, kbs: f32| template.replace("{}", &fmt::rate(kbs));

    row![
        text(format(up, snapshot.system.net_tx_kbs)).size(size),
        style::swatch(EMERALD, 8.0, 4.0),
        text(format(down, snapshot.system.net_rx_kbs)).size(size),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

/// Where CPU time is going, as a colour-coded legend.
fn cpu_usage_section<'a>(snapshot: &'a HardwareSnapshot, is_dark: bool) -> Element<'a, Message> {
    let idle_color = if is_dark {
        Color::from_rgba(1.0, 1.0, 1.0, 0.35)
    } else {
        Color::from_rgba(0.0, 0.0, 0.0, 0.35)
    };

    let entry = |color: Color, label: &'a str, percent: f32| {
        spread_row(
            row![style::swatch(color, 8.0, 2.0), text(label).size(12)]
                .spacing(6)
                .align_y(Alignment::Center),
            text(format!("{percent:.0}%")).size(12),
        )
    };

    column![
        text::title3("CPU Usage").size(13),
        entry(Color::from_rgb(0.23, 0.51, 0.96), "user", snapshot.cpu.user_usage),
        entry(Color::from_rgb(0.94, 0.27, 0.27), "sys", snapshot.cpu.sys_usage),
        entry(idle_color, "idle", snapshot.cpu.idle_usage),
    ]
    .spacing(4)
    .width(Length::FillPortion(1))
    .into()
}

fn load_average_section<'a>(snapshot: &'a HardwareSnapshot) -> Element<'a, Message> {
    let windows = ["1m", "5m", "15m"];

    windows
        .iter()
        .zip(snapshot.system.load_avg)
        .fold(column![text::title3("Load avg").size(13)], |acc, (label, load)| {
            acc.push(spread_row(text(*label).size(12), text(format!("{load:.2}")).size(12)))
        })
        .spacing(4)
        .width(Length::FillPortion(1))
        .into()
}

fn gpu_section<'a>(snapshot: &'a HardwareSnapshot, is_dark: bool) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();

    let content: Element<'a, Message> = match &snapshot.gpu {
        None => text::caption("Integrated graphics or GPU idle").size(11).into(),
        Some(gpu) => {
            let line = |label: &'a str, value: String| {
                spread_row(text::caption(label).size(11), text(value).size(11))
            };

            let mut readings = column![
                line("Memory Used", fmt::bytes(gpu.vram_used_bytes)),
                line("Memory Size", fmt::bytes(gpu.vram_total_bytes)),
            ]
            .spacing(3)
            .width(Length::FillPortion(1));

            if let Some(watts) = gpu.power_draw_watts {
                readings = readings.push(line("Power Draw", format!("{watts:.1} W")));
            }

            let gauge = CircularGauge::new(
                gpu.utilization_percent as f32,
                format!("{}%", gpu.utilization_percent),
                "",
                EMERALD,
            )
            .with_theme(is_dark);

            row![readings, gauge.view(48.0)]
                .spacing(sp.space_m)
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .into()
        }
    };

    column![text::title3("GPU Usage").size(13), content].spacing(6).width(Length::Fill).into()
}

/// Root filesystem usage, with a temperature badge per physical drive.
fn storage_section<'a>(
    snapshot: &'a HardwareSnapshot,
    unit: TemperatureUnit,
) -> Element<'a, Message> {
    let root = snapshot.storage_metrics.root();
    let percent = root.map_or(0.0, |r| r.percent_used);
    let summary = root.map_or_else(String::new, |r| {
        format!("{:.1} GB / {:.1} GB ({percent:.0}%)", fmt::gb(r.used_bytes), fmt::gb(r.total_bytes))
    });

    let heading = row![
        icon::from_name("drive-harddisk-symbolic").size(15),
        text::title3("Storage").size(13),
        cosmic::widget::Space::new().width(Length::Fill),
        text::caption(summary).size(11),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let drives = snapshot.storage.iter().fold(
        column![].spacing(3).width(Length::Fill),
        |list, drive| {
            list.push(spread_row(text(&drive.name).size(12), temp_badge(drive.temp, unit)))
        },
    );

    mouse_area(
        column![heading, determinate_linear(percent / 100.0), drives]
            .spacing(5)
            .width(Length::Fill),
    )
    .on_press(Message::SelectTab(ActiveTab::StorageDetail))
    .interaction(cosmic::iced::mouse::Interaction::Pointer)
    .into()
}

/// A temperature pill coloured by its thermal state.
fn temp_badge<'a>(temp: f32, unit: TemperatureUnit) -> Element<'a, Message> {
    let [r, g, b] = ThermalStatus::from_celsius(temp).rgb();

    container(text(unit.format_short(temp)).size(11))
        .padding([2, 8])
        .class(style::accent_chip(Color::from_rgb(r, g, b)))
        .into()
}

/// Centres a widget across the popup's full width.
fn centered<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content.into()).align_x(Alignment::Center).width(Length::Fill).into()
}
