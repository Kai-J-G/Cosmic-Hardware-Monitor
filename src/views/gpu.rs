//! GPU tab: load, clock, power, temperature and VRAM.

use cosmic::iced::{Color, Length};
use cosmic::widget::{column, container, determinate_linear, row, text};
use cosmic::Element;

use crate::app::Message;
use crate::fl;
use crate::hardware::types::{GpuInfo, TemperatureUnit, ThermalStatus};
use crate::views::{fmt, graph_card, nav_row, label_and_value, stat_card, style};

pub fn view<'a>(
    gpu: Option<&'a GpuInfo>,
    history: &'a [f32],
    unit: TemperatureUnit,
    is_dark: bool,
) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();

    let Some(gpu) = gpu else {
        return unavailable(is_dark);
    };

    let [r, g, b] = ThermalStatus::from_celsius(gpu.edge_temp).rgb();
    let state_color = Color::from_rgb(r, g, b);

    column![
        nav_row(gpu.name.clone()),
        metrics(gpu, unit, state_color, is_dark),
        graph_card(fl!("gpu-temperature-curve"), history, state_color, is_dark),
        vram(gpu, is_dark),
    ]
    .spacing(sp.space_s)
    .width(Length::Fill)
    .into()
}

/// The four headline readings. Values a driver does not expose read as `N/A`
/// rather than disappearing, so the row keeps its shape.
fn metrics<'a>(
    gpu: &'a GpuInfo,
    unit: TemperatureUnit,
    state_color: Color,
    is_dark: bool,
) -> Element<'a, Message> {
    let or_na = |value: Option<String>| value.unwrap_or_else(|| fl!("not-available"));

    row![
        stat_card(
            fl!("utilization"),
            fmt::percent(gpu.utilization_percent as f32, 0),
            None,
            None,
            is_dark,
        ),
        stat_card(fl!("clock-speed"), or_na(gpu.clock_mhz.map(megahertz)), None, None, is_dark),
        stat_card(fl!("power-draw"), or_na(gpu.power_draw_watts.map(watts)), None, None, is_dark),
        stat_card(
            fl!("temperature"),
            unit.format(gpu.edge_temp),
            // The hotspot reading, where the vendor exposes one.
            gpu.junction_temp.map(|j| {
                let temperature = unit.format_short(j);
                fl!("gpu-junction", temperature = temperature.as_str())
            }),
            Some(state_color),
            is_dark,
        ),
    ]
    .spacing(cosmic::theme::spacing().space_s)
    .width(Length::Fill)
    .into()
}

fn vram<'a>(gpu: &'a GpuInfo, is_dark: bool) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();
    let percent = if gpu.vram_total_bytes == 0 {
        0.0
    } else {
        (gpu.vram_used_bytes as f32 / gpu.vram_total_bytes as f32) * 100.0
    };

    let content = column![
        label_and_value(
            text::title3(vram_heading(percent)).size(13),
            text::caption(vram_used_of_total(gpu)).size(11),
        ),
        determinate_linear((percent / 100.0).clamp(0.0, 1.0)),
    ]
    .spacing(sp.space_xs)
    .width(Length::Fill);

    container(content)
        .padding(sp.space_m)
        .width(Length::Fill)
        .class(style::card(is_dark))
        .into()
}

/// `1.72 GB / 15.92 GB`, with both sizes localized.
fn vram_used_of_total(gpu: &GpuInfo) -> String {
    let (used, total) = (fmt::bytes(gpu.vram_used_bytes), fmt::bytes(gpu.vram_total_bytes));
    fl!("vram-used-of-total", used = used.as_str(), total = total.as_str())
}

fn vram_heading(percent: f32) -> String {
    let percent = format!("{percent:.1}");
    fl!("vram-usage", percent = percent.as_str())
}

fn megahertz(value: u32) -> String {
    let value = value.to_string();
    fl!("unit-megahertz", value = value.as_str())
}

fn watts(value: f32) -> String {
    let value = format!("{value:.1}");
    fl!("unit-watts", value = value.as_str())
}

/// Shown when there is no discrete GPU, or no driver telemetry for it.
fn unavailable<'a>(is_dark: bool) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();

    column![
        nav_row(fl!("graphics")),
        container(text::body(fl!("gpu-unavailable")))
            .padding(sp.space_m)
            .width(Length::Fill)
            .class(style::card(is_dark)),
    ]
    .spacing(sp.space_m)
    .width(Length::Fill)
    .into()
}
