//! Helpers for reading Linux `sysfs` and `procfs`.
//!
//! Every value the applet shows comes from a plain text file under `/sys` or
//! `/proc`. All reads here are best-effort: a missing or malformed file gives
//! back `None` rather than an error, because which sensors exist varies wildly
//! between machines and an absent sensor is normal, not a failure.

use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Instant;

/// Reads a file and trims the trailing newline sysfs puts on everything.
pub fn read(path: impl AsRef<Path>) -> Option<String> {
    let contents = fs::read_to_string(path).ok()?;
    Some(contents.trim().to_string())
}

/// Reads a file holding a single value and parses it.
///
/// ```ignore
/// let rpm: u32 = sysfs::parse("/sys/class/hwmon/hwmon0/fan1_input")?;
/// ```
pub fn parse<T: FromStr>(path: impl AsRef<Path>) -> Option<T> {
    read(path)?.parse().ok()
}

/// Reads a value that hwmon stores multiplied by 1000.
///
/// Temperatures are in millidegrees, power in microwatts-per-thousand, and so
/// on; the convention lets the kernel avoid floating point.
fn read_thousandths(path: impl AsRef<Path>) -> Option<f32> {
    let raw: f32 = parse(path)?;
    Some(raw / 1000.0)
}

/// One sensor chip directory under `/sys/class/hwmon`.
///
/// Chips are numbered in discovery order (`hwmon0`, `hwmon1`, ...), which is
/// not stable across reboots, so always find a chip by its `name` — never by
/// its directory number.
pub struct Hwmon {
    /// The driver behind this chip, e.g. `k10temp`, `amdgpu`, `nvme`.
    pub name: String,
    /// The chip's directory, e.g. `/sys/class/hwmon/hwmon2`.
    pub dir: PathBuf,
}

impl Hwmon {
    /// Reads `temp<n>_input` as degrees Celsius.
    ///
    /// Sensor numbering is driver-specific: `amdgpu` uses 1, 2 and 3 for edge,
    /// junction and memory, while a drive's composite temperature is 1.
    pub fn temp(&self, n: u8) -> Option<f32> {
        read_thousandths(self.dir.join(format!("temp{n}_input")))
    }

    /// Every temperature this chip exposes, as `(label, degrees Celsius)`.
    ///
    /// A driver that names its sensors puts the name in `temp<n>_label`
    /// alongside the reading; where it doesn't, the bare `temp<n>` is used.
    pub fn labeled_temps(&self) -> Vec<(String, f32)> {
        let Ok(entries) = fs::read_dir(&self.dir) else {
            return Vec::new();
        };

        let mut temps = Vec::new();

        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let Some(file_name) = file_name.to_str() else {
                continue;
            };

            // Keep only `temp<n>_input` files; `sensor` is then `temp<n>`.
            let Some(sensor) = file_name.strip_suffix("_input") else {
                continue;
            };
            if !sensor.starts_with("temp") {
                continue;
            }

            let Some(celsius) = read_thousandths(entry.path()) else {
                continue;
            };
            let label = read(self.dir.join(format!("{sensor}_label")))
                .unwrap_or_else(|| sensor.to_string());

            temps.push((label, celsius));
        }

        temps
    }
}

/// Lists every hwmon chip on the system.
///
/// The CPU, GPU and storage collectors all need chips from this directory, so
/// [`crate::hardware::HardwareCollector`] scans it once per refresh and lends
/// the result to each of them rather than having all three walk it separately.
pub fn hwmon_chips() -> Vec<Hwmon> {
    let Ok(entries) = fs::read_dir("/sys/class/hwmon") else {
        return Vec::new();
    };

    let mut chips = Vec::new();

    for entry in entries.flatten() {
        let dir = entry.path();
        // A directory with no `name` file isn't a sensor chip we can identify.
        if let Some(name) = read(dir.join("name")) {
            chips.push(Hwmon { name, dir });
        }
    }

    chips
}

/// Converts a kernel byte counter into a transfer rate in KB/s.
///
/// Counters like `/proc/net/dev`'s received bytes only ever grow, so a rate is
/// the difference between two readings divided by the time between them. That
/// means one meter per counter, and no rate at all until the second reading —
/// the first call always returns zero.
#[derive(Default)]
pub struct RateMeter {
    previous_total: u64,
    read_at: Option<Instant>,
}

impl RateMeter {
    /// Records the counter's current total and returns the rate since the
    /// previous call.
    pub fn kb_per_second(&mut self, total_bytes: u64) -> f32 {
        let now = Instant::now();
        let mut rate = 0.0;

        if let Some(previous_read_at) = self.read_at {
            let elapsed = (now - previous_read_at).as_secs_f32();
            // Below ~50ms the division amplifies rounding into a wild figure,
            // so report nothing rather than a spike.
            if elapsed > 0.05 {
                let new_bytes = total_bytes.saturating_sub(self.previous_total);
                rate = new_bytes as f32 / elapsed / 1024.0;
            }
        }

        self.previous_total = total_bytes;
        self.read_at = Some(now);
        rate
    }
}
