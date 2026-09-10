//! Overview tab: four dials, and an expandable panel of detail sections.

use cosmic::iced::{Alignment, Color, Length};
use cosmic::widget::{column, container, determinate_linear, icon, mouse_area, row, text};
use cosmic::Element;

use crate::app::{ActiveTab, Message};
use crate::fl;
use crate::hardware::types::{HardwareSnapshot, TemperatureUnit, ThermalStatus};
use crate::views::circular_gauge::CircularGauge;
use crate::views::{fmt, label_and_value, style};

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
            column![text::caption(fl!("network")).size(11), net_rates(snapshot, NetLabels::Compact)]
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
    let dial = |percent: f32, label: String, sub: Option<String>, tab: ActiveTab| {
        let gauge = CircularGauge::new(percent, fmt::percent(percent, 0), label, style::accent())
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
    let mut dials = row![
        dial(
            snapshot.cpu.overall_usage,
            fl!("cpu"),
            Some(unit.format_short(snapshot.cpu.package_temp)),
            ActiveTab::CpuDetail,
        ),
        dial(gpu_load, fl!("gpu"), gpu_temp, ActiveTab::GpuDetail),
        dial(snapshot.memory.percent, fl!("memory"), None, ActiveTab::MemoryDetail),
    ]
    .spacing(18)
    .align_y(Alignment::Center);

    // Dropped rather than shown at zero where the root filesystem cannot be
    // measured, which is the case inside a Flatpak sandbox.
    if let Some(root) = snapshot.storage_metrics.root() {
        dials = dials.push(dial(root.percent_used, fl!("disk"), None, ActiveTab::StorageDetail));
    }

    dials.into()
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
        label_and_value(
            text(fl!("uptime")).size(12),
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
                text::title3(fl!("network")).size(13),
            ]
            .spacing(6)
            .align_y(Alignment::Center)
        ),
        centered(net_rates(snapshot, NetLabels::Descriptive)),
    ]
    .spacing(4)
    .width(Length::Fill)
    .into()
}

/// How much room the network rates have to explain themselves.
enum NetLabels {
    /// Arrows only, for the collapsed overview.
    Compact,
    /// Spelled out, for the expanded section that has the width for it.
    Descriptive,
}

/// Upload and download rates either side of a status dot.
fn net_rates<'a>(snapshot: &'a HardwareSnapshot, labels: NetLabels) -> Element<'a, Message> {
    let upload = fmt::rate(snapshot.system.net_tx_kbs);
    let download = fmt::rate(snapshot.system.net_rx_kbs);

    let (up, down, size) = match labels {
        NetLabels::Compact => (format!("↑ {upload}"), format!("{download} ↓"), 11),
        NetLabels::Descriptive => (
            format!("{}: ↑ {upload}", fl!("upload")),
            format!("{}: {download} ↓", fl!("download")),
            12,
        ),
    };

    row![
        text(up).size(size),
        style::swatch(style::accent(), 8.0, 4.0),
        text(down).size(size),
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

    let entry = |color: Color, label: String, percent: f32| {
        label_and_value(
            row![style::swatch(color, 8.0, 2.0), text(label).size(12)]
                .spacing(6)
                .align_y(Alignment::Center),
            text(fmt::percent(percent, 0)).size(12),
        )
    };

    column![
        text::title3(fl!("cpu-usage")).size(13),
        entry(Color::from_rgb(0.23, 0.51, 0.96), fl!("usage-user"), snapshot.cpu.user_usage),
        entry(Color::from_rgb(0.94, 0.27, 0.27), fl!("usage-system"), snapshot.cpu.sys_usage),
        entry(idle_color, fl!("usage-idle"), snapshot.cpu.idle_usage),
    ]
    .spacing(4)
    .width(Length::FillPortion(1))
    .into()
}

/// The 1, 5 and 15 minute load averages.
fn load_average_section<'a>(snapshot: &'a HardwareSnapshot) -> Element<'a, Message> {
    let windows = [fl!("load-1m"), fl!("load-5m"), fl!("load-15m")];
    let mut section = column![text::title3(fl!("load-average")).size(13)];

    for (window, load) in windows.into_iter().zip(snapshot.system.load_avg) {
        section = section
            .push(label_and_value(text(window).size(12), text(format!("{load:.2}")).size(12)));
    }

    section.spacing(4).width(Length::FillPortion(1)).into()
}

fn gpu_section<'a>(snapshot: &'a HardwareSnapshot, is_dark: bool) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();

    let content: Element<'a, Message> = match &snapshot.gpu {
        None => text::caption(fl!("gpu-absent")).size(11).into(),
        Some(gpu) => {
            let line = |label: String, value: String| {
                label_and_value(text::caption(label).size(11), text(value).size(11))
            };

            let mut readings = column![
                line(fl!("gpu-memory-used"), fmt::bytes(gpu.vram_used_bytes)),
                line(fl!("gpu-memory-size"), fmt::bytes(gpu.vram_total_bytes)),
            ]
            .spacing(3)
            .width(Length::FillPortion(1));

            if let Some(watts) = gpu.power_draw_watts {
                let draw = format!("{watts:.1}");
                readings = readings
                    .push(line(fl!("power-draw"), fl!("unit-watts", value = draw.as_str())));
            }

            let gauge = CircularGauge::new(
                gpu.utilization_percent as f32,
                fmt::percent(gpu.utilization_percent as f32, 0),
                String::new(),
                style::accent(),
            )
            .with_theme(is_dark);

            row![readings, gauge.view(48.0)]
                .spacing(sp.space_m)
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .into()
        }
    };

    column![text::title3(fl!("gpu-usage")).size(13), content].spacing(6).width(Length::Fill).into()
}

/// Root filesystem usage, with a temperature badge per physical drive.
fn storage_section<'a>(
    snapshot: &'a HardwareSnapshot,
    unit: TemperatureUnit,
) -> Element<'a, Message> {
    let root = snapshot.storage_metrics.root();

    let heading = row![
        icon::from_name("drive-harddisk-symbolic").size(15),
        text::title3(fl!("storage")).size(13),
        cosmic::widget::Space::new().width(Length::Fill),
        text::caption(root.map_or_else(String::new, |r| format!(
            "{:.1} GB / {:.1} GB ({:.0}%)",
            fmt::gb(r.used_bytes),
            fmt::gb(r.total_bytes),
            r.percent_used
        )))
        .size(11),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let mut drives = column![].spacing(3).width(Length::Fill);
    for drive in &snapshot.storage {
        drives =
            drives.push(label_and_value(text(&drive.name).size(12), temp_badge(drive.temp, unit)));
    }

    let mut section = column![heading].spacing(5).width(Length::Fill);

    // Without a measurable root filesystem the bar would sit at zero, so the
    // drive temperatures below stand on their own.
    if let Some(root) = root {
        section = section.push(determinate_linear(root.percent_used / 100.0));
    }

    mouse_area(section.push(drives))
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
