//! CPU tab: headline metrics, a temperature curve and the per-core grid.

use cosmic::iced::{Alignment, Color, Length};
use cosmic::widget::{column, container, mouse_area, row, text};
use cosmic::Element;

use crate::app::Message;
use crate::fl;
use crate::hardware::types::{CpuCoreInfo, CpuInfo, TemperatureUnit, ThermalStatus};
use crate::views::vertical_bar::VerticalBar;
use crate::views::{fmt, graph_card, nav_row, stat_card, style};

/// Temperature range the per-core bars span, in Celsius.
const BAR_MIN_TEMP: f32 = 25.0;
const BAR_TEMP_SPAN: f32 = 60.0;

pub fn view<'a>(
    cpu: &'a CpuInfo,
    history: &'a [f32],
    unit: TemperatureUnit,
    is_dark: bool,
    expanded: bool,
) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();
    let [r, g, b] = ThermalStatus::from_celsius(cpu.package_temp).rgb();
    let state_color = Color::from_rgb(r, g, b);

    let headline = row![
        stat_card(fl!("temperature"), unit.format(cpu.package_temp), None, Some(state_color), is_dark),
        stat_card(
            fl!("load-utilization"),
            fmt::percent(cpu.overall_usage, 1),
            None,
            None,
            is_dark,
        ),
        stat_card(
            fl!("clock-speed"),
            ghz(cpu.avg_freq_mhz),
            None,
            None,
            is_dark,
        ),
    ]
    .spacing(sp.space_s)
    .width(Length::Fill);

    let mut layout = column![
        nav_row(cpu.model.clone()),
        headline,
        telemetry(cpu, is_dark),
        graph_card(fl!("cpu-temperature-curve"), history, state_color, is_dark),
        cores_toggle(cpu.cores.len(), expanded, is_dark),
    ]
    .spacing(sp.space_s)
    .width(Length::Fill);

    if expanded {
        layout = layout.push(cores_grid(&cpu.cores, unit, is_dark));
    }

    layout.into()
}

/// One-line strip of secondary counters.
fn telemetry<'a>(cpu: &'a CpuInfo, is_dark: bool) -> Element<'a, Message> {
    let entry = |label: String, value: String| {
        row![text::caption(label).size(11), text::body(value).size(11)]
            .spacing(4)
            .align_y(Alignment::Center)
    };

    let power = cpu.power_watts.map_or_else(
        || fl!("power-estimating"),
        |w| {
            let value = format!("{w:.1}");
            fl!("unit-watts", value = value.as_str())
        },
    );
    let separator = || text::caption("·").size(11);

    let mut strip =
        row![entry(fl!("power-draw-label"), power)].spacing(8).align_y(Alignment::Center);

    // Omitted rather than shown wrong where the count is unavailable, which is
    // the case inside a Flatpak sandbox.
    if let Some(processes) = cpu.num_processes {
        strip =
            strip.push(separator()).push(entry(fl!("processes-label"), processes.to_string()));
    }

    let strip = strip
        .push(separator())
        .push(entry(fl!("threads-label"), cpu.num_threads.to_string()))
        .push(separator())
        .push(entry(fl!("handles-label"), cpu.num_handles.to_string()));

    container(strip)
        .padding([6, 12])
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .class(style::card(is_dark))
        .into()
}

/// Header that expands and collapses the per-core grid.
fn cores_toggle<'a>(count: usize, expanded: bool, is_dark: bool) -> Element<'a, Message> {
    let label = row![
        text(fl!("all-cores", count = count)).size(12),
        cosmic::widget::Space::new().width(Length::Fill),
        text(if expanded { "⌃" } else { "⌄" }).size(15),
    ]
    .align_y(Alignment::Center);

    mouse_area(
        container(label).padding([8, 14]).width(Length::Fill).class(style::card(is_dark)),
    )
    .on_press(Message::ToggleCpuCores)
    .interaction(cosmic::iced::mouse::Interaction::Pointer)
    .into()
}

/// The cores split across two balanced rows.
///
/// A single row of sixteen cores would leave each card too narrow to read, so
/// the list is halved. An odd count puts the extra card on the top row.
fn cores_grid<'a>(
    cores: &'a [CpuCoreInfo],
    unit: TemperatureUnit,
    is_dark: bool,
) -> Element<'a, Message> {
    let halfway = cores.len().div_ceil(2);
    let (top, bottom) = cores.split_at(halfway);

    column![
        core_row(top, unit, is_dark),
        core_row(bottom, unit, is_dark),
    ]
    .spacing(8)
    .align_x(Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn core_row<'a>(
    cores: &'a [CpuCoreInfo],
    unit: TemperatureUnit,
    is_dark: bool,
) -> Element<'a, Message> {
    let mut cards = row![].spacing(8).width(Length::Fill);

    for core in cores {
        cards = cards.push(core_card(core, unit, is_dark));
    }

    cards.into()
}

/// A clock speed in GHz, from a reading in MHz.
fn ghz(mhz: u32) -> String {
    let value = format!("{:.2}", mhz as f32 / 1000.0);
    fl!("unit-gigahertz", value = value.as_str())
}

/// A single core: a thermal bar with its temperature, load and clock.
fn core_card<'a>(
    core: &'a CpuCoreInfo,
    unit: TemperatureUnit,
    is_dark: bool,
) -> Element<'a, Message> {
    let [r, g, b] = ThermalStatus::from_celsius(core.temp).rgb();
    // A little fill always shows, so a cool core still reads as present.
    let fill = ((core.temp - BAR_MIN_TEMP) / BAR_TEMP_SPAN).clamp(0.08, 1.0);

    let mut content = column![
        text::caption(&core.label).size(10),
        VerticalBar::new(fill, Color::from_rgb(r, g, b), is_dark).view(10.0, 54.0),
        text::title3(unit.format_short(core.temp)).size(13),
        text::caption(fmt::percent(core.usage_percent, 0)).size(10),
    ]
    .spacing(3)
    .align_x(Alignment::Center);

    if let Some(mhz) = core.freq_mhz {
        content = content.push(text::caption(format!("{:.1}G", mhz as f32 / 1000.0)).size(9));
    }

    container(content)
        .padding([6, 4])
        .width(Length::FillPortion(1))
        .align_x(Alignment::Center)
        .class(style::card(is_dark))
        .into()
}
