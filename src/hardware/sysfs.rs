//! Small helpers for reading Linux `sysfs` and `procfs`.
//!
//! Every value the applet shows comes from a plain text file under `/sys` or
//! `/proc`. All reads are best-effort: a missing or malformed file yields
//! `None` rather than an error, because sensor availability varies wildly
//! between machines and a missing sensor is not a failure.

use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Instant;

/// Reads a file and trims surrounding whitespace.
pub fn read(path: impl AsRef<Path>) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

/// Reads a file containing a single value and parses it.
pub fn parse<T: FromStr>(path: impl AsRef<Path>) -> Option<T> {
    read(path)?.parse().ok()
}

/// Reads a hwmon value stored in thousandths (millidegrees, milliwatts, ...).
pub fn milli(path: impl AsRef<Path>) -> Option<f32> {
    parse::<f32>(path).map(|v| v / 1000.0)
}

/// A single sensor chip exposed under `/sys/class/hwmon`.
pub struct Hwmon {
    /// Driver name, e.g. `k10temp`, `amdgpu`, `nvme`.
    pub name: String,
    pub dir: PathBuf,
}

impl Hwmon {
    /// Reads `<dir>/temp<n>_input` as degrees Celsius.
    pub fn temp(&self, n: u8) -> Option<f32> {
        milli(self.dir.join(format!("temp{n}_input")))
    }

    /// Yields every `temp*_input` in this chip as `(label, celsius)`.
    ///
    /// The label comes from the sibling `temp*_label` file when the driver
    /// provides one, otherwise it falls back to the bare `tempN` prefix.
    pub fn labeled_temps(&self) -> Vec<(String, f32)> {
        let Ok(entries) = fs::read_dir(&self.dir) else {
            return Vec::new();
        };

        entries
            .flatten()
            .filter_map(|entry| {
                let file = entry.file_name();
                let file = file.to_str()?;
                let prefix = file.strip_suffix("_input")?;
                if !prefix.starts_with("temp") {
                    return None;
                }
                let celsius = milli(entry.path())?;
                let label = read(self.dir.join(format!("{prefix}_label")))
                    .unwrap_or_else(|| prefix.to_string());
                Some((label, celsius))
            })
            .collect()
    }
}

/// Enumerates every hwmon chip once, so collectors can share a single scan
/// instead of walking `/sys/class/hwmon` each.
pub fn hwmon_chips() -> Vec<Hwmon> {
    let Ok(entries) = fs::read_dir("/sys/class/hwmon") else {
        return Vec::new();
    };

    entries
        .flatten()
        .filter_map(|entry| {
            let dir = entry.path();
            let name = read(dir.join("name"))?;
            Some(Hwmon { name, dir })
        })
        .collect()
}

/// Turns a pair of monotonically increasing byte counters into KB/s.
///
/// Kernel counters are cumulative, so a rate only exists once we have two
/// samples; the first call always reports zero.
#[derive(Default)]
pub struct RateMeter {
    last: (u64, u64),
    at: Option<Instant>,
}

impl RateMeter {
    pub fn sample(&mut self, a: u64, b: u64) -> (f32, f32) {
        let now = Instant::now();
        let rates = match self.at {
            // Guard against a near-zero interval blowing the rate up.
            Some(prev) if (now - prev).as_secs_f32() > 0.05 => {
                let dt = (now - prev).as_secs_f32();
                let kbs = |cur: u64, prev: u64| (cur.saturating_sub(prev) as f32 / dt) / 1024.0;
                (kbs(a, self.last.0), kbs(b, self.last.1))
            }
            _ => (0.0, 0.0),
        };

        self.last = (a, b);
        self.at = Some(now);
        rates
    }
}
