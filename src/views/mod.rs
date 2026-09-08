pub mod circular_gauge;
pub mod cores;
pub mod fans;
pub mod gpu;
pub mod header;
pub mod hero;
pub mod memory;
pub mod overview;
pub mod settings;
pub mod storage;
pub mod system;
pub mod tabs;
pub mod vertical_bar;

use cosmic::iced::Length;
use cosmic::widget::column;
use cosmic::Element;

use crate::app::{ActiveTab, Message};
use crate::config::TempTyleConfig;
use crate::hardware::types::{HardwareSnapshot, TemperatureUnit};

pub fn popup_width(active_tab: ActiveTab) -> f32 {
    match active_tab {
        ActiveTab::CpuDetail => 760.0,
        ActiveTab::GpuDetail => 740.0,
        ActiveTab::MemoryDetail | ActiveTab::StorageDetail => 640.0,
        ActiveTab::Settings => 480.0,
        ActiveTab::Overview => 440.0,
    }
}

pub fn view_popup<'a>(
    snapshot: &'a HardwareSnapshot,
    cpu_history: &'a [f32],
    _cpu_load_history: &'a [f32],
    gpu_temp_history: &'a [f32],
    _gpu_load_history: &'a [f32],
    mem_history: &'a [f32],
    active_tab: ActiveTab,
    expanded_details: bool,
    expanded_cpu_cores: bool,
    unit: TemperatureUnit,
    is_dark: bool,
    config: &'a TempTyleConfig,
) -> Element<'a, Message> {
    let sp = cosmic::theme::spacing();
    let current_width = popup_width(active_tab);

    // 1. Header (Centered "Hardware Monitor", Settings Button)
    let header = header::view_header(active_tab);

    // 2. Main Content Area (Overview or Drill-down Details)
    let tab_content: Element<'a, Message> = match active_tab {
        ActiveTab::Overview => overview::view_overview(snapshot, expanded_details, unit, is_dark),
        ActiveTab::CpuDetail => cores::view_cores(&snapshot.cpu, cpu_history, unit, is_dark, expanded_cpu_cores),
        ActiveTab::GpuDetail => gpu::view_gpu(snapshot.gpu.as_ref(), gpu_temp_history, unit, is_dark),
        ActiveTab::StorageDetail => storage::view_storage(&snapshot.storage_metrics, is_dark),
        ActiveTab::MemoryDetail => memory::view_memory(&snapshot.memory, mem_history, is_dark),
        ActiveTab::Settings => settings::view_settings(config.theme_pref, unit, config.refresh_interval_secs, is_dark),
    };

    let body = column![header, tab_content]
        .spacing(sp.space_m)
        .padding([sp.space_s, sp.space_m, sp.space_m, sp.space_m])
        .width(Length::Fixed(current_width));

    body.into()
}
