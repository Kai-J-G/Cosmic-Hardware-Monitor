//! CPU model, temperatures, clocks and power draw.
//!
//! Utilisation is the one reading that needs history rather than a single
//! sample, so it lives in [`usage`].

mod usage;

use std::fs;
use std::time::Instant;

use usage::{Usage, UsageTracker};

use super::sysfs::{self, Hwmon};
use super::types::{CpuCoreInfo, CpuInfo};

/// Fallback package temperature when no CPU sensor is exposed at all.
const DEFAULT_PACKAGE_TEMP: f32 = 40.0;

/// hwmon drivers that expose CPU temperatures.
const CPU_CHIPS: [&str; 3] = ["k10temp", "zenpower", "coretemp"];

/// Reads CPU state, holding on to whatever must survive between refreshes:
/// the utilisation counters, the energy counter, and the model name that is
/// only worth reading once.
pub struct CpuCollector {
    /// `/proc/cpuinfo` cannot change while we run, so it is parsed once.
    model: String,
    usage: UsageTracker,
    /// The last energy reading and when it was taken, for the power figure.
    previous_energy: Option<(u64, Instant)>,
}

impl CpuCollector {
    pub fn new() -> Self {
        Self {
            model: read_model(),
            usage: UsageTracker::default(),
            previous_energy: None,
        }
    }

    pub fn collect(&mut self, chips: &[Hwmon]) -> CpuInfo {
        let usage = self.usage.read();
        let (package_temp, core_temps) = read_temps(chips);
        let package_temp = package_temp.unwrap_or(DEFAULT_PACKAGE_TEMP);
        let cores = build_cores(&core_temps, &usage, package_temp);

        let avg_freq_mhz = average_frequency(&cores);
        let (num_processes, num_threads) = read_process_counts();

        CpuInfo {
            model: self.model.clone(),
            package_temp,
            cores,
            overall_usage: usage.overall,
            user_usage: usage.user,
            sys_usage: usage.system,
            idle_usage: usage.idle,
            avg_freq_mhz,
            power_watts: self.read_power(),
            num_processes,
            num_threads,
            num_handles: read_open_handles(),
        }
    }

    /// Package power draw, derived from the kernel's RAPL energy counter.
    ///
    /// The counter accumulates microjoules, so power is the energy used since
    /// the last reading divided by the time it took. Returns `None` when the
    /// counter is absent, or is root-only as some distributions configure it.
    fn read_power(&mut self) -> Option<f32> {
        const RAPL: &str = "/sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj";

        let energy_uj: u64 = sysfs::parse(RAPL)?;
        let now = Instant::now();
        let watts = self.watts_since_last_reading(energy_uj, now);

        self.previous_energy = Some((energy_uj, now));
        watts
    }

    fn watts_since_last_reading(&self, energy_uj: u64, now: Instant) -> Option<f32> {
        let (previous_uj, previous_at) = self.previous_energy?;

        // Too soon to divide by: the result would be dominated by rounding.
        let elapsed = (now - previous_at).as_secs_f32();
        if elapsed <= 0.05 {
            return None;
        }

        let joules = energy_uj.saturating_sub(previous_uj) as f32 / 1_000_000.0;
        let watts = joules / elapsed;

        // The counter is finite and wraps back to zero, which shows up as one
        // absurd reading. Drop anything no desktop CPU could really draw.
        if !(0.0..=500.0).contains(&watts) {
            return None;
        }

        // One decimal place is all the UI shows.
        Some((watts * 10.0).round() / 10.0)
    }
}

/// Splits CPU hwmon sensors into the package reading and the per-core ones.
///
/// AMD (`k10temp`/`zenpower`) reports `Tctl`/`Tdie` plus one `Tccd` per die,
/// Intel (`coretemp`) reports `Package id 0` plus one `Core N` each; both are
/// distinguished by sensor label rather than by index.
fn read_temps(chips: &[Hwmon]) -> (Option<f32>, Vec<(String, f32)>) {
    let mut package = None;
    let mut cores = Vec::new();

    for chip in chips.iter().filter(|c| CPU_CHIPS.contains(&c.name.as_str())) {
        for (label, temp) in chip.labeled_temps() {
            let key = label.to_lowercase();
            if key.starts_with("tctl") || key.starts_with("tdie") || key.contains("package") {
                package = Some(temp);
            } else if key.contains("tccd") || key.contains("core") {
                cores.push((label, temp));
            }
        }
    }

    // Directory order is arbitrary, so sort by the index in the label
    // ("Core 2" before "Core 10", which a plain string sort would invert).
    cores.sort_by_key(|(label, _)| (trailing_number(label), label.clone()));
    (package, cores)
}

/// The number a sensor label ends with — `9` for `Tccd9`, `10` for `Core 10`.
///
/// Returns `None` for a label with no trailing number, such as `Tctl`.
fn trailing_number(label: &str) -> Option<u32> {
    let digits: String = label
        .chars()
        .rev()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .chars()
        .rev()
        .collect();

    digits.parse().ok()
}

/// Builds the per-core list, one entry per online CPU.
///
/// Sensors rarely map one-to-one onto logical CPUs — AMD exposes one sensor
/// per die, not per thread — so a dedicated reading is only used when there
/// are at least as many sensors as CPUs. Otherwise every core shares the best
/// available temperature.
fn build_cores(temps: &[(String, f32)], usage: &Usage, package_temp: f32) -> Vec<CpuCoreInfo> {
    let cpus = online_cpus();
    let per_sensor = temps.len() >= cpus;
    let shared = temps.first().map_or(package_temp, |(_, t)| *t);

    (0..cpus)
        .map(|idx| {
            let (label, temp) = match temps.get(idx) {
                Some((label, temp)) if per_sensor => (label.clone(), *temp),
                _ => (format!("Core {idx}"), shared),
            };
            CpuCoreInfo {
                label,
                temp,
                usage_percent: usage.per_core.get(&idx).copied().unwrap_or(usage.overall),
                freq_mhz: read_core_freq(idx),
            }
        })
        .collect()
}

/// Mean clock speed across the cores that report one, or 0 if none do.
fn average_frequency(cores: &[CpuCoreInfo]) -> u32 {
    let frequencies: Vec<u32> = cores.iter().filter_map(|core| core.freq_mhz).collect();

    if frequencies.is_empty() {
        return 0;
    }

    // Sum as u64: sixteen cores at 5 GHz would not overflow u32, but summing
    // in the wider type costs nothing and removes the question.
    let total: u64 = frequencies.iter().map(|&mhz| u64::from(mhz)).sum();
    (total / frequencies.len() as u64) as u32
}

/// The CPU's marketing name, from the first `model name` line of
/// `/proc/cpuinfo` (every core repeats the same value).
fn read_model() -> String {
    const UNKNOWN: &str = "Processor";

    let Some(contents) = sysfs::read("/proc/cpuinfo") else {
        return UNKNOWN.to_string();
    };
    let Some(line) = contents.lines().find(|line| line.starts_with("model name")) else {
        return UNKNOWN.to_string();
    };

    match line.split_once(':') {
        Some((_key, model)) => model.trim().to_string(),
        None => UNKNOWN.to_string(),
    }
}

/// How many logical CPUs are online.
///
/// `/sys/devices/system/cpu/online` holds a range like `0-15`, so the highest
/// index plus one is the count.
fn online_cpus() -> usize {
    let Some(range) = sysfs::read("/sys/devices/system/cpu/online") else {
        return 1;
    };
    let Some((_first, last)) = range.split_once('-') else {
        // A single-CPU machine reports just "0" with no range.
        return 1;
    };

    match last.parse::<usize>() {
        Ok(highest_index) => highest_index + 1,
        Err(_) => 1,
    }
}

/// A core's current clock speed in MHz.
///
/// `None` when the kernel exposes no scaling driver for it, which is normal on
/// virtual machines.
fn read_core_freq(core: usize) -> Option<u32> {
    let path = format!("/sys/devices/system/cpu/cpu{core}/cpufreq/scaling_cur_freq");
    let kilohertz: u32 = sysfs::parse(path)?;

    Some(kilohertz / 1000)
}

/// Counts running processes and threads.
fn read_process_counts() -> (usize, usize) {
    // Every process has a directory in /proc named after its PID; the other
    // entries there are files and named subsystems, so count numeric names.
    let processes = match fs::read_dir("/proc") {
        Ok(entries) => entries
            .flatten()
            .filter(|entry| entry.file_name().to_string_lossy().parse::<u32>().is_ok())
            .count(),
        Err(_) => 0,
    };

    (processes, read_thread_count())
}

/// Total threads, from the `running/total` field of `/proc/loadavg`.
///
/// A line reads `3.52 2.27 1.89 2/2189 90312`, so the fourth field's second
/// half is what we want.
fn read_thread_count() -> usize {
    let Some(contents) = sysfs::read("/proc/loadavg") else {
        return 0;
    };
    let Some(entities) = contents.split_whitespace().nth(3) else {
        return 0;
    };
    let Some((_running, total)) = entities.split_once('/') else {
        return 0;
    };

    total.parse().unwrap_or(0)
}

/// Open file descriptors system-wide — Linux's equivalent of Windows handles,
/// which is the figure this row of the UI mirrors.
///
/// `/proc/sys/fs/file-nr` holds `allocated  free  maximum`.
fn read_open_handles() -> usize {
    let Some(contents) = sysfs::read("/proc/sys/fs/file-nr") else {
        return 0;
    };
    let Some(allocated) = contents.split_whitespace().next() else {
        return 0;
    };

    allocated.parse().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_core_labels_numerically() {
        let mut labels = ["Core 10", "Core 2", "Core 1", "Tccd9", "Package id 0", "Unlabelled"];
        labels.sort_by_key(|label| (trailing_number(label), *label));

        // Labels without a number sort first; the rest go in numeric order,
        // which a plain string sort would get wrong for "Core 10".
        assert_eq!(
            labels,
            ["Unlabelled", "Package id 0", "Core 1", "Core 2", "Tccd9", "Core 10"]
        );
    }

    #[test]
    fn averages_only_the_cores_reporting_a_frequency() {
        let core = |freq_mhz| CpuCoreInfo {
            label: "Core".to_string(),
            temp: 40.0,
            usage_percent: 0.0,
            freq_mhz,
        };

        assert_eq!(average_frequency(&[core(Some(3000)), core(Some(4000))]), 3500);
        assert_eq!(average_frequency(&[core(Some(3000)), core(None)]), 3000);
        assert_eq!(average_frequency(&[core(None)]), 0);
    }
}
