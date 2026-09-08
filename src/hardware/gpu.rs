//! GPU telemetry for AMD (sysfs), NVIDIA (`nvidia-smi`) and Intel (hwmon).
//!
//! The three vendors expose completely different interfaces, so this module
//! works out which one applies at start-up and then reads only that one. That
//! matters for more than tidiness: naming a GPU can mean running `lspci`, and
//! doing so on every refresh would spawn a process twice a second.

use std::path::{Path, PathBuf};
use std::process::Command;

use super::sysfs::{self, Hwmon};
use super::types::GpuInfo;

/// Shown when a GPU is present but exposes no temperature sensor.
const DEFAULT_EDGE_TEMP: f32 = 35.0;

/// The interface this machine's GPU is read through, chosen once at start-up.
enum Source {
    /// AMD reports temperatures through hwmon, and load and VRAM through the
    /// card's DRM device directory.
    Amd {
        /// `/sys/class/drm/card*/device`, if a card exposing load was found.
        drm: Option<PathBuf>,
    },
    /// NVIDIA publishes nothing useful through sysfs; `nvidia-smi`, which
    /// ships with the proprietary driver, is the supported interface.
    Nvidia,
    /// Intel integrated graphics expose a temperature and nothing else.
    Intel,
}

pub struct GpuCollector {
    /// `None` when no supported GPU was found; the GPU tab then says so.
    source: Option<Source>,
    /// Resolved once, because it cannot change while the applet runs.
    name: String,
}

impl GpuCollector {
    /// Detects the GPU. Vendors are tried discrete-first, since a machine with
    /// both will want the card that does the work.
    pub fn new(chips: &[Hwmon]) -> Self {
        if find_chip(chips, is_amd).is_some() {
            let drm = find_drm_device();
            return Self { name: amd_name(drm.as_deref()), source: Some(Source::Amd { drm }) };
        }

        if let Some(name) = nvidia_name() {
            return Self { source: Some(Source::Nvidia), name };
        }

        if find_chip(chips, is_intel).is_some() {
            let name = lspci_name("Intel").unwrap_or_else(|| "Intel Graphics".to_string());
            return Self { source: Some(Source::Intel), name };
        }

        Self { source: None, name: String::new() }
    }

    pub fn collect(&self, chips: &[Hwmon]) -> Option<GpuInfo> {
        match self.source.as_ref()? {
            Source::Amd { drm } => Some(self.read_amd(chips, drm.as_deref())),
            Source::Nvidia => self.read_nvidia(),
            Source::Intel => Some(self.read_intel(chips)),
        }
    }

    fn read_amd(&self, chips: &[Hwmon], drm: Option<&Path>) -> GpuInfo {
        let hwmon = find_chip(chips, is_amd);

        GpuInfo {
            // amdgpu numbers its sensors edge, junction (the hotspot, always
            // the hotter of the two), then memory.
            edge_temp: hwmon.and_then(|chip| chip.temp(1)).unwrap_or(DEFAULT_EDGE_TEMP),
            junction_temp: hwmon.and_then(|chip| chip.temp(2)),
            power_draw_watts: hwmon.and_then(amd_power_watts),
            clock_mhz: hwmon.and_then(amd_clock_mhz).or_else(|| drm.and_then(current_sclk)),
            utilization_percent: read_drm_value(drm, "gpu_busy_percent").unwrap_or(0).min(100) as u32,
            vram_used_bytes: read_drm_value(drm, "mem_info_vram_used").unwrap_or(0),
            vram_total_bytes: read_drm_value(drm, "mem_info_vram_total").unwrap_or(0),
            name: self.name.clone(),
        }
    }

    fn read_nvidia(&self) -> Option<GpuInfo> {
        let readings = nvidia_query(&[
            "temperature.gpu",
            "utilization.gpu",
            "memory.used",
            "memory.total",
            "power.draw",
            "clocks.current.graphics",
        ])?;

        // Values come back in the order requested, as bare numbers. A field
        // the card doesn't support reads "N/A" and so parses to None.
        let vram_mb = |index| reading::<u64>(&readings, index).unwrap_or(0) * 1024 * 1024;

        Some(GpuInfo {
            edge_temp: reading(&readings, 0).unwrap_or(DEFAULT_EDGE_TEMP),
            junction_temp: None,
            utilization_percent: reading(&readings, 1).unwrap_or(0),
            vram_used_bytes: vram_mb(2),
            vram_total_bytes: vram_mb(3),
            power_draw_watts: reading(&readings, 4),
            clock_mhz: reading(&readings, 5),
            name: self.name.clone(),
        })
    }

    fn read_intel(&self, chips: &[Hwmon]) -> GpuInfo {
        GpuInfo {
            edge_temp: find_chip(chips, is_intel)
                .and_then(|chip| chip.temp(1))
                .unwrap_or(DEFAULT_EDGE_TEMP),
            // Integrated graphics share system memory and publish no load
            // counter, so the rest of the tab stays empty.
            junction_temp: None,
            utilization_percent: 0,
            clock_mhz: None,
            vram_used_bytes: 0,
            vram_total_bytes: 0,
            power_draw_watts: None,
            name: self.name.clone(),
        }
    }
}

/// Parses one value out of an `nvidia-smi` row.
fn reading<T: std::str::FromStr>(readings: &[String], index: usize) -> Option<T> {
    readings.get(index)?.parse().ok()
}

fn is_amd(driver: &str) -> bool {
    driver == "amdgpu"
}

/// `i915` drives older Intel graphics, `xe` the newer ones.
fn is_intel(driver: &str) -> bool {
    driver.contains("i915") || driver.contains("xe")
}

fn find_chip<'a>(chips: &'a [Hwmon], matches: fn(&str) -> bool) -> Option<&'a Hwmon> {
    chips.iter().find(|chip| matches(&chip.name))
}

/// AMD power draw in watts, reported by hwmon in microwatts.
///
/// `power1_average` is smoothed over a short window and is what a monitoring
/// tool wants; `power1_input` is the instantaneous reading some cards offer
/// instead.
fn amd_power_watts(chip: &Hwmon) -> Option<f32> {
    let microwatts: f32 = sysfs::parse(chip.dir.join("power1_average"))
        .or_else(|| sysfs::parse(chip.dir.join("power1_input")))?;

    Some(microwatts / 1_000_000.0)
}

/// AMD shader clock in MHz, reported by hwmon in hertz.
fn amd_clock_mhz(chip: &Hwmon) -> Option<u32> {
    let hertz: u64 = sysfs::parse(chip.dir.join("freq1_input"))?;

    Some((hertz / 1_000_000) as u32)
}

fn read_drm_value(drm: Option<&Path>, file: &str) -> Option<u64> {
    sysfs::parse(drm?.join(file))
}

/// Finds the DRM device belonging to the real GPU.
///
/// `/sys/class/drm` also lists display connectors and render nodes; only the
/// card itself publishes a load counter, which makes that file a reliable way
/// to tell them apart.
fn find_drm_device() -> Option<PathBuf> {
    let entries = std::fs::read_dir("/sys/class/drm").ok()?;

    entries
        .flatten()
        .map(|entry| entry.path().join("device"))
        .find(|device| device.join("gpu_busy_percent").exists())
}

/// Reads the active entry from AMD's shader clock table.
///
/// `pp_dpm_sclk` lists every clock state, with the current one marked:
///
/// ```text
/// 0: 500Mhz
/// 1: 897Mhz *
/// 2: 2970Mhz
/// ```
fn current_sclk(drm: &Path) -> Option<u32> {
    let table = sysfs::read(drm.join("pp_dpm_sclk"))?;
    let active = table.lines().find(|line| line.ends_with('*'))?;
    let speed = active.split_whitespace().nth(1)?;

    // Strip the "Mhz" suffix, whose capitalisation varies by driver version.
    speed.trim_end_matches(char::is_alphabetic).parse().ok()
}

/// The GPU's name, preferring what the driver advertises over `lspci`.
fn amd_name(drm: Option<&Path>) -> String {
    let from_driver = drm
        .and_then(|device| sysfs::read(device.join("product_name")))
        .filter(|name| !name.is_empty());

    from_driver
        .or_else(|| lspci_name("AMD"))
        .unwrap_or_else(|| "AMD Radeon GPU".to_string())
}

/// The NVIDIA GPU's name, which doubles as the test for whether one is here:
/// no driver means no `nvidia-smi` to run.
fn nvidia_name() -> Option<String> {
    nvidia_query(&["name"])?.into_iter().next()
}

/// Runs one `nvidia-smi` query and returns its comma-separated values.
///
/// `noheader` drops the column titles and `nounits` drops the suffixes, so the
/// output is a single line of bare numbers matching the requested fields.
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
    let values: Vec<String> =
        stdout.lines().next()?.split(',').map(|value| value.trim().to_string()).collect();

    // A short row means the driver didn't understand a field we asked for.
    if values.len() != fields.len() {
        return None;
    }

    Some(values)
}

/// Falls back to `lspci` for a readable name when sysfs offers none.
///
/// Requires `pciutils`; without it the caller's generic name is used.
fn lspci_name(vendor: &str) -> Option<String> {
    let output = Command::new("lspci").output().ok()?;
    if !output.status.success() {
        return None;
    }

    let listing = String::from_utf8_lossy(&output.stdout);
    let line = listing
        .lines()
        .filter(|line| line.contains("VGA compatible controller") || line.contains("3D controller"))
        .find(|line| line.contains(vendor))?;
    let (_address, description) = line.split_once(": ")?;

    Some(marketing_name(description.trim()))
}

/// Reduces an `lspci` description to the name a person would recognise.
///
/// Descriptions are verbose:
///
/// ```text
/// Advanced Micro Devices, Inc. [AMD/ATI] Navi 48 [Radeon RX 9070 XT] (rev c0)
///                                                 ^ the retail name
/// ```
///
/// The retail name is the last bracketed group. Where there isn't one, the
/// revision suffix is the only part worth dropping.
fn marketing_name(description: &str) -> String {
    if let Some((_before, tail)) = description.rsplit_once('[') {
        if let Some((name, _after)) = tail.split_once(']') {
            return name.trim().to_string();
        }
    }

    match description.split_once(" (rev ") {
        Some((name, _revision)) => name.trim().to_string(),
        None => description.trim().to_string(),
    }
}
