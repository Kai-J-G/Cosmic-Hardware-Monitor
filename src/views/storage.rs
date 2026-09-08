//! Storage tab: disk throughput and per-partition usage.

use cosmic::iced::{Alignment, Color, Length};
use cosmic::widget::{column, container, determinate_linear, icon, row, scrollable, text};
use cosmic::Element;

use crate::app::Message;
use crate::hardware::types::{PartitionInfo, StorageMetrics};
use crate::views::{fmt, nav_row, label_and_value, style, EMERALD};

const CYAN: Color = Color::from_rgb(0.0, 0.75, 1.0);

/// Height of the partition list before it starts scrolling.
const LIST_HEIGHT: f32 = 260.0;

pub fn view<'a>(metrics: &'a StorageMetrics, is_dark: bool) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();

    let speeds = row![
        speed_card("Read Speed", metrics.read_kbs, "go-up-symbolic", CYAN, is_dark),
        speed_card("Write Speed", metrics.write_kbs, "go-down-symbolic", EMERALD, is_dark),
    ]
    .spacing(sp.space_s)
    .width(Length::Fill);

    let partitions: Element<'a, Message> = if metrics.partitions.is_empty() {
        // Inside a Flatpak sandbox the mount table describes the sandbox, so
        // the list is deliberately empty; say so rather than look broken.
        let reason = if crate::hardware::sandbox::is_flatpak() {
            "Partition usage is unavailable in the Flatpak build, which cannot see \
             the host's filesystems. Throughput above is still accurate."
        } else {
            "No mounted partitions detected."
        };
        text::caption(reason).size(12).into()
    } else {
        let mut list = column![].spacing(sp.space_xs).width(Length::Fill);
        for partition in &metrics.partitions {
            list = list.push(partition_card(partition, is_dark));
        }
        list.into()
    };

    column![
        nav_row("Storage & Partition Metrics"),
        speeds,
        text::title3("Mounted Partitions & Free Space").size(13),
        scrollable(partitions).height(Length::Fixed(LIST_HEIGHT)).width(Length::Fill),
    ]
    .spacing(sp.space_m)
    .width(Length::Fill)
    .into()
}

/// Throughput in one direction, tinted with its own accent.
fn speed_card<'a>(
    label: &'a str,
    kbs: f32,
    icon_name: &'a str,
    accent: Color,
    is_dark: bool,
) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();
    let content = row![
        icon::from_name(icon_name).size(18),
        column![text::caption(label).size(11), text::title3(fmt::rate(kbs)).size(17)].spacing(2),
    ]
    .spacing(sp.space_s)
    .align_y(Alignment::Center);

    container(content)
        .padding([10, 14])
        .width(Length::FillPortion(1))
        .class(style::accent_card(accent, is_dark))
        .into()
}

/// One mount point: where it is, how full it is, and how much is left.
fn partition_card<'a>(part: &'a PartitionInfo, is_dark: bool) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();

    let heading = row![
        icon::from_name("drive-harddisk-symbolic").size(18),
        text::body(format!("Mount: {}", part.mount)).size(13),
        container(text::caption(&part.filesystem).size(10))
            .padding([2, 6])
            .class(style::chip(is_dark)),
        cosmic::widget::Space::new().width(Length::Fill),
        text::title3(format!("{:.1}% used", part.percent_used)).size(13),
    ]
    .spacing(sp.space_xs)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let footer = label_and_value(
        text::body(format!("Free: {}", fmt::bytes(part.free_bytes))).size(11),
        text::caption(format!(
            "{} used of {}",
            fmt::bytes(part.used_bytes),
            fmt::bytes(part.total_bytes)
        ))
        .size(11),
    );

    container(
        column![heading, determinate_linear((part.percent_used / 100.0).clamp(0.0, 1.0)), footer]
            .spacing(5)
            .width(Length::Fill),
    )
    .padding([sp.space_s, sp.space_m])
    .width(Length::Fill)
    .class(style::card(is_dark))
    .into()
}
