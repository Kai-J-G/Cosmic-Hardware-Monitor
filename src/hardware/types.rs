//! The data the collectors produce and the views render.
//!
//! These are plain structs with no behaviour beyond a few conversions. Adding
//! a reading to the UI generally means adding a field here, filling it in the
//! matching collector, and displaying it in the matching view.
//!
//! Units are named in every field, because the kernel's own units vary: bytes
//! here, kibibytes there, millidegrees elsewhere. Everything below is already
//! converted into the unit its name states.

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

/// The band a temperature falls into, which decides the colour the UI shows.
///
/// The thresholds suit a desktop CPU or GPU under load; drives and memory run
/// far cooler and so sit permanently in the lower bands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalStatus {
    /// Below 45°C — idle or lightly loaded.
    Cool,
    /// 45–64°C — a normal working temperature.
    Optimal,
    /// 65–74°C — sustained load.
    Normal,
    /// 75–84°C — hot, but within spec for most parts.
    Warm,
    /// 85°C and above — at or near thermal throttling.
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

/// One logical CPU.
#[derive(Debug, Clone)]
pub struct CpuCoreInfo {
    /// The sensor's own name where there is one per core (`Core 3`, `Tccd1`),
    /// otherwise a generated `Core N`.
    pub label: String,
    /// Degrees Celsius. Cores sharing a die sensor all report the same value.
    pub temp: f32,
    /// 0–100.
    pub usage_percent: f32,
    /// Current clock in MHz, or `None` where no scaling driver exists.
    pub freq_mhz: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct CpuInfo {
    /// Marketing name, e.g. `AMD Ryzen 7 7800X3D 8-Core Processor`.
    pub model: String,
    /// Whole-package temperature in Celsius — the headline CPU reading.
    pub package_temp: f32,
    /// One entry per online logical CPU.
    pub cores: Vec<CpuCoreInfo>,
    /// 0–100, across all cores. Equal to `100 - idle_usage`.
    pub overall_usage: f32,
    /// 0–100, time spent running user programs.
    pub user_usage: f32,
    /// 0–100, time spent in the kernel and servicing interrupts.
    pub sys_usage: f32,
    /// 0–100, time spent idle or waiting on I/O.
    pub idle_usage: f32,
    /// Mean clock across cores reporting one, in MHz; 0 if none do.
    pub avg_freq_mhz: u32,
    /// Package draw in watts, or `None` where the RAPL counter is unreadable.
    pub power_watts: Option<f32>,
    pub num_processes: usize,
    pub num_threads: usize,
    /// Open file descriptors system-wide.
    pub num_handles: usize,
}

/// Fields are `Option` wherever a vendor may not report them: Intel
/// integrated graphics, for instance, expose only a temperature.
#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub name: String,
    /// Board temperature in Celsius, the figure vendors quote.
    pub edge_temp: f32,
    /// Hotspot temperature in Celsius — always the hotter of the two.
    pub junction_temp: Option<f32>,
    /// 0–100.
    pub utilization_percent: u32,
    /// Shader clock in MHz.
    pub clock_mhz: Option<u32>,
    /// Dedicated video memory. Zero on integrated graphics, which use system
    /// memory instead.
    pub vram_used_bytes: u64,
    pub vram_total_bytes: u64,
    pub power_draw_watts: Option<f32>,
}

/// The temperature of one physical drive.
#[derive(Debug, Clone)]
pub struct StorageInfo {
    /// The drive's model where the kernel knows it, e.g. `NVMe Samsung SSD
    /// 990 EVO 2TB`.
    pub name: String,
    /// Composite temperature in Celsius.
    pub temp: f32,
}

/// One mounted filesystem.
#[derive(Debug, Clone)]
pub struct PartitionInfo {
    /// Where it is mounted, e.g. `/` or `/home`.
    pub mount: String,
    /// Its type, e.g. `btrfs`, `ext4`, `vfat`.
    pub filesystem: String,
    pub total_bytes: u64,
    /// Space a normal user can still fill; excludes root-reserved blocks, so
    /// `free_bytes + used_bytes` can be slightly less than `total_bytes`.
    pub free_bytes: u64,
    pub used_bytes: u64,
    /// 0–100.
    pub percent_used: f32,
}

#[derive(Debug, Clone, Default)]
pub struct StorageMetrics {
    /// Throughput across all whole disks, in KB/s.
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
    /// Total minus available, which is what people mean by "in use".
    pub used_bytes: u64,
    /// What a new program could claim, counting reclaimable cache.
    pub available_bytes: u64,
    /// Memory programs have asked for, which can exceed the total because
    /// Linux hands out more than it has on the assumption it won't all be
    /// touched.
    pub committed_bytes: u64,
    /// Page cache and block buffers combined — in use, but reclaimable.
    pub cached_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub swap_avail_bytes: u64,
    /// Physical memory in use, 0–100.
    pub percent: f32,
}

#[derive(Debug, Clone, Default)]
pub struct SystemInfo {
    pub uptime_seconds: u64,
    /// Load averaged over 1, 5 and 15 minutes. This counts processes waiting
    /// to run, so it is not a percentage and can exceed the number of cores.
    pub load_avg: [f32; 3],
    /// Throughput across all interfaces but loopback, in KB/s.
    pub net_rx_kbs: f32,
    pub net_tx_kbs: f32,
}

/// One complete reading of the machine, rebuilt on every refresh tick.
///
/// The views render this and nothing else, which is what keeps them free of
/// their own state.
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
