use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemperatureUnit {
    Celsius,
    Fahrenheit,
}

impl TemperatureUnit {
    pub fn format_temp(self, celsius: f32) -> String {
        match self {
            Self::Celsius => format!("{:.0}°C", celsius),
            Self::Fahrenheit => format!("{:.0}°F", (celsius * 9.0 / 5.0) + 32.0),
        }
    }

    pub fn format_temp_val(self, celsius: f32) -> (String, &'static str) {
        match self {
            Self::Celsius => (format!("{:.0}", celsius), "°C"),
            Self::Fahrenheit => (format!("{:.0}", (celsius * 9.0 / 5.0) + 32.0), "°F"),
        }
    }

    pub fn format_temp_short(self, celsius: f32) -> String {
        match self {
            Self::Celsius => format!("{:.0}°", celsius),
            Self::Fahrenheit => format!("{:.0}°", (celsius * 9.0 / 5.0) + 32.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThermalStatus {
    Cool,     // < 45°C
    Optimal,  // 45°C - 64°C
    Normal,   // 65°C - 74°C
    Warm,     // 75°C - 84°C
    Critical, // 85°C+
}

impl ThermalStatus {
    pub fn from_celsius(temp: f32) -> Self {
        if temp < 45.0 {
            Self::Cool
        } else if temp < 65.0 {
            Self::Optimal
        } else if temp < 75.0 {
            Self::Normal
        } else if temp < 85.0 {
            Self::Warm
        } else {
            Self::Critical
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Cool => "Cool Thermal State",
            Self::Optimal => "Optimal Thermal State",
            Self::Normal => "Normal Thermal State",
            Self::Warm => "Warm Thermal State",
            Self::Critical => "Critical Thermal State",
        }
    }

    /// [r, g, b] color components in 0.0..1.0
    pub fn rgb(self) -> [f32; 3] {
        match self {
            Self::Cool => [0.0, 0.75, 1.0],       // #00B0FF Neon Cyan
            Self::Optimal => [0.0, 0.90, 0.46],   // #00E676 Vibrant Emerald
            Self::Normal => [1.0, 0.75, 0.0],     // #FFC107 Warm Gold
            Self::Warm => [1.0, 0.55, 0.0],       // #FF8C00 Amber
            Self::Critical => [1.0, 0.30, 0.30],   // #FF4D4D Coral Red
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalSummary {
    pub package_temp: f32,
    pub gpu_temp: Option<f32>,
    pub max_temp: f32,
    pub min_temp: f32,
    pub avg_temp: f32,
    pub total_cores: usize,
    pub active_fans_count: usize,
    pub thermal_status: ThermalStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuCoreInfo {
    pub id: usize,
    pub label: String,
    pub temp: f32,
    pub usage_percent: f32,
    pub freq_mhz: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub name: String,
    pub cpu_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub model: String,
    pub package_temp: f32,
    pub cores: Vec<CpuCoreInfo>,
    pub overall_usage: f32,
    pub user_usage: f32,
    pub sys_usage: f32,
    pub idle_usage: f32,
    pub avg_freq_mhz: u32,
    pub power_watts: Option<f32>,
    pub num_processes: usize,
    pub num_threads: usize,
    pub num_handles: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub edge_temp: f32,
    pub junction_temp: Option<f32>,
    pub mem_temp: Option<f32>,
    pub utilization_percent: u32,
    pub clock_mhz: Option<u32>,
    pub vram_used_bytes: u64,
    pub vram_total_bytes: u64,
    pub power_draw_watts: Option<f32>,
    pub power_cap_watts: Option<f32>,
    pub fan_rpm: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FanInfo {
    pub chip: String,
    pub name: String,
    pub rpm: u32,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub name: String,
    pub composite_temp: f32,
    pub sensor1_temp: Option<f32>,
    pub sensor2_temp: Option<f32>,
    pub crit_temp: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionInfo {
    pub mount: String,
    pub filesystem: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub used_bytes: u64,
    pub percent_used: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageMetrics {
    pub read_kbs: f32,
    pub write_kbs: f32,
    pub partitions: Vec<PartitionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryMetrics {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub committed_bytes: u64,
    pub cached_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub swap_avail_bytes: u64,
    pub percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os_name: String,
    pub kernel: String,
    pub uptime_seconds: u64,
    pub load_avg: [f32; 3],
    pub mem_used_bytes: u64,
    pub mem_total_bytes: u64,
    pub mem_percent: f32,
    pub spd_temps: Vec<(String, f32)>,
    pub disk_used_bytes: u64,
    pub disk_total_bytes: u64,
    pub disk_percent: f32,
    pub net_rx_kbs: f32,
    pub net_tx_kbs: f32,
    pub top_processes: Vec<ProcessInfo>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryPoint {
    pub temp: f32,
    pub gpu_temp: Option<f32>,
    pub cpu_load: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSnapshot {
    pub summary: ThermalSummary,
    pub cpu: CpuInfo,
    pub gpu: Option<GpuInfo>,
    pub fans: Vec<FanInfo>,
    pub storage: Vec<StorageInfo>,
    pub storage_metrics: StorageMetrics,
    pub memory: MemoryMetrics,
    pub system: SystemInfo,
}
