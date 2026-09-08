//! Plain data the collectors produce and the views render.

/// Unit the user has chosen for every temperature in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemperatureUnit {
    Celsius,
    Fahrenheit,
}

impl TemperatureUnit {
    /// Converts a Celsius reading into this unit.
    fn convert(self, celsius: f32) -> f32 {
        match self {
            Self::Celsius => celsius,
            Self::Fahrenheit => (celsius * 9.0 / 5.0) + 32.0,
        }
    }

    /// Full reading with its unit, e.g. `62°C`.
    pub fn format(self, celsius: f32) -> String {
        let symbol = match self {
            Self::Celsius => "C",
            Self::Fahrenheit => "F",
        };
        format!("{:.0}°{symbol}", self.convert(celsius))
    }

    /// Degree-only reading for tight spots such as the panel, e.g. `62°`.
    pub fn format_short(self, celsius: f32) -> String {
        format!("{:.0}°", self.convert(celsius))
    }
}

/// Coarse thermal band a reading falls into, used to colour the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalStatus {
    Cool,
    Optimal,
    Normal,
    Warm,
    Critical,
}

impl ThermalStatus {
    pub fn from_celsius(temp: f32) -> Self {
        match temp {
            t if t < 45.0 => Self::Cool,
            t if t < 65.0 => Self::Optimal,
            t if t < 75.0 => Self::Normal,
            t if t < 85.0 => Self::Warm,
            _ => Self::Critical,
        }
    }

    /// Accent colour for this band as `[r, g, b]` in `0.0..=1.0`.
    pub fn rgb(self) -> [f32; 3] {
        match self {
            Self::Cool => [0.00, 0.75, 1.00],     // cyan
            Self::Optimal => [0.00, 0.90, 0.46],  // emerald
            Self::Normal => [1.00, 0.75, 0.00],   // gold
            Self::Warm => [1.00, 0.55, 0.00],     // amber
            Self::Critical => [1.00, 0.30, 0.30], // coral
        }
    }
}

#[derive(Debug, Clone)]
pub struct CpuCoreInfo {
    pub label: String,
    pub temp: f32,
    pub usage_percent: f32,
    pub freq_mhz: Option<u32>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub name: String,
    pub edge_temp: f32,
    pub junction_temp: Option<f32>,
    pub utilization_percent: u32,
    pub clock_mhz: Option<u32>,
    pub vram_used_bytes: u64,
    pub vram_total_bytes: u64,
    pub power_draw_watts: Option<f32>,
}

/// Temperature of one physical drive.
#[derive(Debug, Clone)]
pub struct StorageInfo {
    pub name: String,
    pub temp: f32,
}

#[derive(Debug, Clone)]
pub struct PartitionInfo {
    pub mount: String,
    pub filesystem: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub used_bytes: u64,
    pub percent_used: f32,
}

#[derive(Debug, Clone, Default)]
pub struct StorageMetrics {
    pub read_kbs: f32,
    pub write_kbs: f32,
    /// Mounted partitions, root first.
    pub partitions: Vec<PartitionInfo>,
}

impl StorageMetrics {
    /// The root filesystem, which stands in for "the disk" on the overview.
    pub fn root(&self) -> Option<&PartitionInfo> {
        self.partitions.iter().find(|p| p.mount == "/")
    }
}

#[derive(Debug, Clone, Default)]
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

#[derive(Debug, Clone, Default)]
pub struct SystemInfo {
    pub uptime_seconds: u64,
    pub load_avg: [f32; 3],
    pub net_rx_kbs: f32,
    pub net_tx_kbs: f32,
}

/// One complete reading of the machine, rebuilt on every refresh tick.
#[derive(Debug, Clone)]
pub struct HardwareSnapshot {
    pub cpu: CpuInfo,
    pub gpu: Option<GpuInfo>,
    pub storage: Vec<StorageInfo>,
    pub storage_metrics: StorageMetrics,
    pub memory: MemoryMetrics,
    pub system: SystemInfo,
}

impl HardwareSnapshot {
    /// The reading the panel badge reflects: whichever of CPU package and GPU
    /// edge is currently hotter.
    pub fn primary_temp(&self) -> f32 {
        match &self.gpu {
            Some(gpu) => self.cpu.package_temp.max(gpu.edge_temp),
            None => self.cpu.package_temp,
        }
    }

    pub fn status(&self) -> ThermalStatus {
        ThermalStatus::from_celsius(self.primary_temp())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_both_units() {
        assert_eq!(TemperatureUnit::Celsius.format(62.4), "62°C");
        assert_eq!(TemperatureUnit::Celsius.format_short(62.4), "62°");
        assert_eq!(TemperatureUnit::Fahrenheit.format(100.0), "212°F");
        assert_eq!(TemperatureUnit::Fahrenheit.format(0.0), "32°F");
    }

    #[test]
    fn thermal_bands_cover_their_boundaries() {
        assert_eq!(ThermalStatus::from_celsius(44.9), ThermalStatus::Cool);
        assert_eq!(ThermalStatus::from_celsius(45.0), ThermalStatus::Optimal);
        assert_eq!(ThermalStatus::from_celsius(65.0), ThermalStatus::Normal);
        assert_eq!(ThermalStatus::from_celsius(75.0), ThermalStatus::Warm);
        assert_eq!(ThermalStatus::from_celsius(85.0), ThermalStatus::Critical);
    }

    #[test]
    fn root_partition_is_found_wherever_it_sits() {
        let partition = |mount: &str| PartitionInfo {
            mount: mount.to_string(),
            filesystem: "btrfs".to_string(),
            total_bytes: 100,
            free_bytes: 40,
            used_bytes: 60,
            percent_used: 60.0,
        };

        let metrics = StorageMetrics {
            partitions: vec![partition("/boot"), partition("/")],
            ..Default::default()
        };

        assert_eq!(metrics.root().map(|r| r.mount.as_str()), Some("/"));
        assert!(StorageMetrics::default().root().is_none());
    }
}
