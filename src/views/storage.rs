//! Storage tab: disk throughput and per-partition usage.

use cosmic::iced::{Alignment, Color, Length};
use cosmic::widget::{column, container, determinate_linear, icon, row, scrollable, text};
use cosmic::Element;

use crate::app::Message;
use crate::fl;
use crate::hardware::types::{PartitionInfo, StorageMetrics};
use crate::views::{fmt, nav_row, style, EMERALD};

const CYAN: Color = Color::from_rgb(0.0, 0.75, 1.0);

/// How many partitions the list will grow to fit before it starts scrolling.
///
/// The popup sizes itself to its content, so the usual handful of mounts all
/// show at once with no scrollbar. Past this many the list would grow taller
/// than a screen, so it scrolls instead.
const MAX_UNSCROLLED_PARTITIONS: usize = 12;

/// Height the list is capped at once it does scroll.
const SCROLLED_LIST_HEIGHT: f32 = 480.0;

pub fn view<'a>(metrics: &'a StorageMetrics, is_dark: bool) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();

    let speeds = row![
        speed_card(fl!("read-speed"), metrics.read_kbs, "go-up-symbolic", CYAN, is_dark),
        speed_card(fl!("write-speed"), metrics.write_kbs, "go-down-symbolic", EMERALD, is_dark),
    ]
    .spacing(sp.space_s)
    .width(Length::Fill);

    let partitions: Element<'a, Message> = if metrics.partitions.is_empty() {
        // Inside a Flatpak sandbox the mount table describes the sandbox, so
        // the list is deliberately empty; say so rather than look broken.
        let reason = if crate::hardware::sandbox::is_flatpak() {
            fl!("partitions-sandboxed")
        } else {
            fl!("partitions-none")
        };
        text::caption(reason).size(12).into()
    } else {
        let mut list = column![].spacing(sp.space_xs).width(Length::Fill);
        for partition in &metrics.partitions {
            list = list.push(partition_card(partition, is_dark));
        }
        list.into()
    };

    // Only impose a scrollbar on the machines that genuinely need one.
    let partitions: Element<'a, Message> = if metrics.partitions.len() > MAX_UNSCROLLED_PARTITIONS {
        scrollable(partitions).height(Length::Fixed(SCROLLED_LIST_HEIGHT)).width(Length::Fill).into()
    } else {
        partitions
    };

    column![
        nav_row(fl!("storage-title")),
        speeds,
        text::title3(fl!("partitions-title")).size(13),
        partitions,
    ]
    .spacing(sp.space_m)
    .width(Length::Fill)
    .into()
}

/// Throughput in one direction, tinted with its own accent.
fn speed_card<'a>(
    label: String,
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

/// The right-hand summary on a partition row.
fn partition_summary(part: &PartitionInfo) -> String {
    let percent = format!("{:.0}", part.percent_used);
    let (free, total) = (fmt::bytes(part.free_bytes), fmt::bytes(part.total_bytes));
    fl!(
        "partition-summary",
        percent = percent.as_str(),
        free = free.as_str(),
        total = total.as_str()
    )
}

/// One mount point: where it is, how full it is, and how much is left.
///
/// Deliberately two rows rather than three. The list has to fit however many
/// disks the machine has without scrolling, so every row costs height across
/// the whole list.
fn partition_card<'a>(part: &'a PartitionInfo, is_dark: bool) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();

    let heading = row![
        icon::from_name("drive-harddisk-symbolic").size(14),
        text::body(&part.mount).size(12),
        container(text::caption(&part.filesystem).size(9))
            .padding([1, 5])
            .class(style::chip(is_dark)),
        cosmic::widget::Space::new().width(Length::Fill),
        text::caption(partition_summary(part)).size(11),
    ]
    .spacing(sp.space_xxs)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    container(
        column![heading, determinate_linear((part.percent_used / 100.0).clamp(0.0, 1.0))]
            .spacing(4)
            .width(Length::Fill),
    )
    .padding([sp.space_xxs, sp.space_s])
    .width(Length::Fill)
    .class(style::card(is_dark))
    .into()
}
