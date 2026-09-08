//! GPU tab: load, clock, power, temperature and VRAM.

use cosmic::iced::{Color, Length};
use cosmic::widget::{column, container, determinate_linear, row, text};
use cosmic::Element;

use crate::app::Message;
use crate::hardware::types::{GpuInfo, TemperatureUnit, ThermalStatus};
use crate::views::{fmt, graph_card, nav_row, spread_row, stat_card, style};

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
        graph_card("GPU Temperature Curve", history, state_color, is_dark),
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
    let or_na = |value: Option<String>| value.unwrap_or_else(|| "N/A".to_string());

    row![
        stat_card("Utilization", format!("{}%", gpu.utilization_percent), None, None, is_dark),
        stat_card(
            "Clock Speed",
            or_na(gpu.clock_mhz.map(|mhz| format!("{mhz} MHz"))),
            None,
            None,
            is_dark,
        ),
        stat_card(
            "Power Draw",
            or_na(gpu.power_draw_watts.map(|w| format!("{w:.1} W"))),
            None,
            None,
            is_dark,
        ),
        stat_card(
            "Temperature",
            unit.format(gpu.edge_temp),
            // The hotspot reading, where the vendor exposes one.
            gpu.junction_temp.map(|j| format!("Junction: {}", unit.format_short(j))),
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
        spread_row(
            text::title3(format!("VRAM Usage: {percent:.1}%")).size(13),
            text::caption(format!(
                "{:.2} GB / {:.2} GB",
                fmt::gb(gpu.vram_used_bytes),
                fmt::gb(gpu.vram_total_bytes)
            ))
            .size(11),
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

/// Shown when there is no discrete GPU, or no driver telemetry for it.
fn unavailable<'a>(is_dark: bool) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();

    column![
        nav_row("Graphics"),
        container(text::body("No dedicated GPU detected or GPU metrics unavailable."))
            .padding(sp.space_m)
            .width(Length::Fill)
            .class(style::card(is_dark)),
    ]
    .spacing(sp.space_m)
    .width(Length::Fill)
    .into()
}
