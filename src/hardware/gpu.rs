//! GPU telemetry for AMD (sysfs), NVIDIA (`nvidia-smi`) and Intel (hwmon).

use std::path::PathBuf;
use std::process::Command;

use super::sysfs::{self, Hwmon};
use super::types::GpuInfo;

/// Fallback edge temperature when the vendor exposes no thermal sensor.
const DEFAULT_EDGE_TEMP: f32 = 35.0;

/// Which vendor path we read each tick.
///
/// Probing is deliberately done once at start-up: identifying the GPU can mean
/// spawning `lspci`, and the answer cannot change while the applet runs.
enum Source {
    /// AMD exposes temperatures via hwmon and load/VRAM via the DRM device.
    Amd { drm: Option<PathBuf> },
    /// NVIDIA has no sysfs telemetry; `nvidia-smi` is the supported interface.
    Nvidia,
    /// Intel integrated graphics only expose a temperature.
    Intel,
}

pub struct GpuCollector {
    source: Option<Source>,
    name: String,
}

impl GpuCollector {
    pub fn new(chips: &[Hwmon]) -> Self {
        if has_chip(chips, |n| n == "amdgpu") {
            let drm = find_drm_device();
            let name = drm
                .as_ref()
                .and_then(|d| sysfs::read(d.join("product_name")))
                .filter(|n| !n.is_empty())
                .or_else(|| lspci_name("AMD"))
                .unwrap_or_else(|| "AMD Radeon GPU".to_string());
            return Self { source: Some(Source::Amd { drm }), name };
        }

        if let Some(name) = nvidia_query(&["name"]).and_then(|f| f.into_iter().next()) {
            return Self { source: Some(Source::Nvidia), name };
        }

        if has_chip(chips, |n| n.contains("i915") || n.contains("xe")) {
            let name = lspci_name("Intel").unwrap_or_else(|| "Intel Graphics".to_string());
            return Self { source: Some(Source::Intel), name };
        }

        Self { source: None, name: String::new() }
    }

    pub fn collect(&self, chips: &[Hwmon]) -> Option<GpuInfo> {
        match self.source.as_ref()? {
            Source::Amd { drm } => Some(self.read_amd(chips, drm.as_deref())),
            Source::Nvidia => self.read_nvidia(),
            Source::Intel => Some(GpuInfo {
                edge_temp: chip(chips, |n| n.contains("i915") || n.contains("xe"))
                    .and_then(|c| c.temp(1))
                    .unwrap_or(DEFAULT_EDGE_TEMP),
                ..self.blank()
            }),
        }
    }

    fn read_amd(&self, chips: &[Hwmon], drm: Option<&std::path::Path>) -> GpuInfo {
        let hwmon = chip(chips, |n| n == "amdgpu");
        // temp1/2/3 are edge, junction (hotspot) and memory respectively.
        let edge_temp = hwmon.and_then(|c| c.temp(1)).unwrap_or(DEFAULT_EDGE_TEMP);
        let junction_temp = hwmon.and_then(|c| c.temp(2));

        // Reported in microwatts. `power1_average` is smoothed and preferred;
        // `power1_input` is the instantaneous reading some cards expose instead.
        let power_draw_watts = hwmon.and_then(|c| {
            sysfs::parse::<f32>(c.dir.join("power1_average"))
                .or_else(|| sysfs::parse::<f32>(c.dir.join("power1_input")))
                .map(|microwatts| microwatts / 1_000_000.0)
        });

        // Some cards report the shader clock through hwmon, others only via DRM.
        let clock_mhz = hwmon
            .and_then(|c| sysfs::parse::<u64>(c.dir.join("freq1_input")))
            .map(|hz| (hz / 1_000_000) as u32)
            .or_else(|| drm.and_then(current_sclk));

        GpuInfo {
            edge_temp,
            junction_temp,
            clock_mhz,
            power_draw_watts,
            utilization_percent: drm
                .and_then(|d| sysfs::parse::<u32>(d.join("gpu_busy_percent")))
                .unwrap_or(0)
                .min(100),
            vram_used_bytes: drm
                .and_then(|d| sysfs::parse(d.join("mem_info_vram_used")))
                .unwrap_or(0),
            vram_total_bytes: drm
                .and_then(|d| sysfs::parse(d.join("mem_info_vram_total")))
                .unwrap_or(0),
            ..self.blank()
        }
    }

    fn read_nvidia(&self) -> Option<GpuInfo> {
        let fields = nvidia_query(&[
            "temperature.gpu",
            "utilization.gpu",
            "memory.used",
            "memory.total",
            "power.draw",
            "clocks.current.graphics",
        ])?;
        // `nounits` output is plain numbers, in the order requested above.
        let mib = |i| field::<u64>(&fields, i).unwrap_or(0) * 1024 * 1024;

        Some(GpuInfo {
            edge_temp: field(&fields, 0).unwrap_or(DEFAULT_EDGE_TEMP),
            utilization_percent: field(&fields, 1).unwrap_or(0),
            vram_used_bytes: mib(2),
            vram_total_bytes: mib(3),
            power_draw_watts: field(&fields, 4),
            clock_mhz: field(&fields, 5),
            ..self.blank()
        })
    }

    /// An otherwise-empty reading carrying the GPU's name.
    fn blank(&self) -> GpuInfo {
        GpuInfo {
            name: self.name.clone(),
            edge_temp: DEFAULT_EDGE_TEMP,
            junction_temp: None,
            utilization_percent: 0,
            clock_mhz: None,
            vram_used_bytes: 0,
            vram_total_bytes: 0,
            power_draw_watts: None,
        }
    }
}

/// Parses the `i`th field of an `nvidia-smi` row; `N/A` values parse to `None`.
fn field<T: std::str::FromStr>(fields: &[String], i: usize) -> Option<T> {
    fields.get(i)?.parse().ok()
}

fn chip(chips: &[Hwmon], matches: impl Fn(&str) -> bool) -> Option<&Hwmon> {
    chips.iter().find(|c| matches(&c.name))
}

fn has_chip(chips: &[Hwmon], matches: impl Fn(&str) -> bool) -> bool {
    chip(chips, matches).is_some()
}

/// The first DRM device exposing load metrics, i.e. the real GPU rather than
/// a display-only node.
fn find_drm_device() -> Option<PathBuf> {
    std::fs::read_dir("/sys/class/drm")
        .ok()?
        .flatten()
        .map(|e| e.path().join("device"))
        .find(|device| device.join("gpu_busy_percent").exists())
}

/// Reads the active entry (marked `*`) from the AMD shader clock table.
fn current_sclk(drm: &std::path::Path) -> Option<u32> {
    sysfs::read(drm.join("pp_dpm_sclk"))?
        .lines()
        .find(|l| l.ends_with('*'))?
        .split_whitespace()
        .nth(1)?
        .trim_end_matches(|c: char| c.is_ascii_alphabetic())
        .parse()
        .ok()
}

/// Runs one `nvidia-smi` query and returns its comma-separated fields.
fn nvidia_query(fields: &[&str]) -> Option<Vec<String>> {
    let output = Command::new("nvidia-smi")
        .arg(format!("--query-gpu={}", fields.join(",")))
        .arg("--format=csv,noheader,nounits")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let values: Vec<String> = stdout
        .lines()
        .next()?
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    (values.len() == fields.len()).then_some(values)
}

/// Falls back to `lspci` for a human-readable name when sysfs has none.
fn lspci_name(vendor: &str) -> Option<String> {
    let output = Command::new("lspci").output().ok()?;
    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| l.contains("VGA compatible controller") || l.contains("3D controller"))
        .find(|l| l.contains(vendor))
        .and_then(|l| l.split_once(": "))
        .map(|(_, description)| marketing_name(description.trim()))
}

/// Reduces an `lspci` description to the name a person would recognise.
///
/// Descriptions read like `Advanced Micro Devices, Inc. [AMD/ATI] Navi 48
/// [Radeon RX 9070 XT] (rev c0)`, where the retail name is the last bracketed
/// group. Failing that, the revision suffix is all that is worth dropping.
fn marketing_name(description: &str) -> String {
    description
        .rsplit_once('[')
        .and_then(|(_, tail)| tail.split_once(']'))
        .map(|(name, _)| name)
        .unwrap_or_else(|| description.split_once(" (rev ").map_or(description, |(name, _)| name))
        .trim()
        .to_string()
}
