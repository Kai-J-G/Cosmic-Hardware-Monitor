//! Memory tab: RAM in use, history and the swap breakdown.

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{column, container, determinate_linear, row, text};
use cosmic::Element;

use crate::app::Message;
use crate::fl;
use crate::hardware::types::MemoryMetrics;
use crate::views::{fmt, graph_card, nav_row, label_and_value, stat_card, style, EMERALD};

pub fn view<'a>(
    memory: &'a MemoryMetrics,
    history: &'a [f32],
    is_dark: bool,
) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();

    column![
        nav_row(fl!("memory-title")),
        headline(memory, is_dark),
        graph_card(fl!("memory-history"), history, EMERALD, is_dark),
        usage_bar(memory),
        details(memory, is_dark),
    ]
    .spacing(sp.space_m)
    .width(Length::Fill)
    .into()
}

/// The two figures that answer "how much memory is left?".
fn headline<'a>(memory: &'a MemoryMetrics, is_dark: bool) -> Element<'a, Message> {
    row![
        stat_card(
            fl!("memory-in-use"),
            fmt::bytes(memory.used_bytes),
            Some(percent_of_ram(memory.percent)),
            None,
            is_dark,
        ),
        stat_card(
            fl!("memory-available"),
            fmt::bytes(memory.available_bytes),
            Some(percent_free(percent_of(memory.available_bytes, memory.total_bytes))),
            None,
            is_dark,
        ),
    ]
    .spacing(cosmic::theme::spacing().space_s)
    .width(Length::Fill)
    .into()
}

/// Overall physical memory pressure, as a bar with its own summary line.
fn usage_bar<'a>(memory: &'a MemoryMetrics) -> Element<'a, Message> {
    column![
        label_and_value(
            text::title3(physical_memory(memory.percent)).size(13),
            text::caption(in_use_of_total(memory.used_bytes, memory.total_bytes)).size(11),
        ),
        determinate_linear((memory.percent / 100.0).clamp(0.0, 1.0)),
    ]
    .spacing(cosmic::theme::spacing().space_xs)
    .width(Length::Fill)
    .into()
}

/// The secondary figures: what is committed, cached and swapped.
fn details<'a>(memory: &'a MemoryMetrics, is_dark: bool) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();
    let swap_used_pct = percent_of(memory.swap_used_bytes, memory.swap_total_bytes);

    column![
        row![
            detail(fl!("memory-committed"), fmt::bytes(memory.committed_bytes), None, is_dark),
            detail(fl!("memory-cached"), fmt::bytes(memory.cached_bytes), None, is_dark),
        ]
        .spacing(sp.space_s),
        row![
            detail(
                fl!("swap-used"),
                fmt::bytes(memory.swap_used_bytes),
                Some(swap_percent_used(swap_used_pct)),
                is_dark,
            ),
            detail(
                fl!("swap-available"),
                fmt::bytes(memory.swap_avail_bytes),
                Some(swap_of_total(memory.swap_total_bytes)),
                is_dark,
            ),
        ]
        .spacing(sp.space_s),
    ]
    .spacing(sp.space_s)
    .width(Length::Fill)
    .into()
}

/// A secondary reading, left-aligned and smaller than the headline cards.
fn detail<'a>(
    label: String,
    value: String,
    sub: Option<String>,
    is_dark: bool,
) -> Element<'a, Message> {
    let mut content =
        column![text::caption(label).size(11), text::title3(value).size(15)].spacing(2);
    if let Some(sub) = sub {
        content = content.push(text::caption(sub).size(10));
    }

    container(content)
        .padding([8, 12])
        .width(Length::FillPortion(1))
        .align_y(Alignment::Center)
        .class(style::card(is_dark))
        .into()
}

fn percent_of_ram(percent: f32) -> String {
    let percent = format!("{percent:.1}");
    fl!("memory-percent-of-ram", percent = percent.as_str())
}

fn percent_free(percent: f32) -> String {
    let percent = format!("{percent:.1}");
    fl!("memory-percent-free", percent = percent.as_str())
}

fn physical_memory(percent: f32) -> String {
    let percent = format!("{percent:.1}");
    fl!("physical-memory", percent = percent.as_str())
}

fn in_use_of_total(used_bytes: u64, total_bytes: u64) -> String {
    let (used, total) = (fmt::bytes(used_bytes), fmt::bytes(total_bytes));
    fl!("memory-in-use-of-total", used = used.as_str(), total = total.as_str())
}

fn swap_percent_used(percent: f32) -> String {
    let percent = format!("{percent:.1}");
    fl!("swap-percent-used", percent = percent.as_str())
}

fn swap_of_total(total_bytes: u64) -> String {
    let total = fmt::bytes(total_bytes);
    fl!("swap-of-total", total = total.as_str())
}

/// Guards against the divide-by-zero a machine without swap would produce.
fn percent_of(part: u64, whole: u64) -> f32 {
    if whole == 0 {
        0.0
    } else {
        (part as f32 / whole as f32) * 100.0
    }
}
