//! CPU model, per-core temperature/frequency, utilisation and power draw.

use std::collections::HashMap;
use std::fs;
use std::time::Instant;

use super::sysfs::{self, Hwmon};
use super::types::{CpuCoreInfo, CpuInfo};

/// Fallback package temperature when no CPU sensor is exposed at all.
const DEFAULT_PACKAGE_TEMP: f32 = 40.0;

/// hwmon drivers that expose CPU temperatures.
const CPU_CHIPS: [&str; 3] = ["k10temp", "zenpower", "coretemp"];

/// Reads CPU state, carrying the counters that only make sense as a delta
/// between two ticks (utilisation and power both derive from totals).
pub struct CpuCollector {
    /// `/proc/cpuinfo` never changes while we run, so parse it once.
    model: String,
    prev_overall: Option<CpuTimes>,
    prev_cores: HashMap<usize, CpuTimes>,
    prev_energy: Option<(u64, Instant)>,
}

impl CpuCollector {
    pub fn new() -> Self {
        Self {
            model: read_model(),
            prev_overall: None,
            prev_cores: HashMap::new(),
            prev_energy: None,
        }
    }

    pub fn collect(&mut self, chips: &[Hwmon]) -> CpuInfo {
        let usage = self.read_usage();
        let (package_temp, core_temps) = read_temps(chips);
        let package_temp = package_temp.unwrap_or(DEFAULT_PACKAGE_TEMP);
        let cores = build_cores(&core_temps, &usage, package_temp);

        let freqs: Vec<u32> = cores.iter().filter_map(|c| c.freq_mhz).collect();
        let avg_freq_mhz = match freqs.len() {
            0 => 0,
            n => (freqs.iter().map(|&f| u64::from(f)).sum::<u64>() / n as u64) as u32,
        };

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

    /// Turns the cumulative jiffy counters in `/proc/stat` into percentages.
    fn read_usage(&mut self) -> Usage {
        let Some(content) = sysfs::read("/proc/stat") else {
            return Usage::default();
        };

        let mut usage = Usage::default();

        for line in content.lines() {
            let mut fields = line.split_whitespace();
            let Some(key) = fields.next() else { continue };
            let Some(rest) = key.strip_prefix("cpu") else {
                // `/proc/stat` lists the cpu lines first; nothing after matters.
                break;
            };
            let Some(times) = CpuTimes::parse(fields) else {
                continue;
            };

            if rest.is_empty() {
                // The aggregate "cpu" line: the only one we break down by kind.
                if let Some(prev) = self.prev_overall.replace(times) {
                    usage = times.usage_since(prev);
                }
            } else if let Ok(core) = rest.parse::<usize>() {
                if let Some(prev) = self.prev_cores.insert(core, times) {
                    usage.per_core.insert(core, times.usage_since(prev).overall);
                }
            }
        }

        usage
    }

    /// Derives package power from the RAPL energy counter, when present.
    fn read_power(&mut self) -> Option<f32> {
        const RAPL: &str = "/sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj";

        let energy_uj: u64 = sysfs::parse(RAPL)?;
        let now = Instant::now();
        let watts = self.prev_energy.and_then(|(prev_uj, prev_at)| {
            let dt = (now - prev_at).as_secs_f32();
            if dt <= 0.05 {
                return None;
            }
            let watts = (energy_uj.saturating_sub(prev_uj) as f32 / 1_000_000.0) / dt;
            // The counter wraps; reject the implausible spike that produces.
            (0.0..=500.0).contains(&watts).then(|| (watts * 10.0).round() / 10.0)
        });

        self.prev_energy = Some((energy_uj, now));
        watts
    }
}

/// Jiffies spent in each state, as reported by one `/proc/stat` cpu line.
#[derive(Default, Clone, Copy)]
struct CpuTimes {
    user: u64,
    system: u64,
    idle: u64,
}

impl CpuTimes {
    /// Parses `user nice system idle iowait irq softirq steal ...`.
    fn parse<'a>(fields: impl Iterator<Item = &'a str>) -> Option<Self> {
        let v: Vec<u64> = fields.take(8).map(|f| f.parse().unwrap_or(0)).collect();
        if v.len() < 4 {
            return None;
        }
        let at = |i: usize| v.get(i).copied().unwrap_or(0);
        Some(Self {
            user: at(0) + at(1),                     // user + nice
            system: at(2) + at(5) + at(6) + at(7),   // system + irq + softirq + steal
            idle: at(3) + at(4),                     // idle + iowait
        })
    }

    fn total(self) -> u64 {
        self.user + self.system + self.idle
    }

    fn usage_since(self, prev: Self) -> Usage {
        let total = self.total().saturating_sub(prev.total());
        if total == 0 {
            return Usage::default();
        }
        let pct = |cur: u64, old: u64| (cur.saturating_sub(old) as f32 / total as f32) * 100.0;
        let idle = pct(self.idle, prev.idle);
        Usage {
            overall: (100.0 - idle).clamp(0.0, 100.0),
            user: pct(self.user, prev.user),
            system: pct(self.system, prev.system),
            idle,
            per_core: HashMap::new(),
        }
    }
}

/// Utilisation percentages for one tick.
struct Usage {
    overall: f32,
    user: f32,
    system: f32,
    idle: f32,
    per_core: HashMap<usize, f32>,
}

impl Default for Usage {
    fn default() -> Self {
        Self { overall: 0.0, user: 0.0, system: 0.0, idle: 100.0, per_core: HashMap::new() }
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

/// The number a sensor label ends with, e.g. `9` for `Tccd9`, or `None`.
fn trailing_number(label: &str) -> Option<u32> {
    let digits = label.trim_end_matches(|c: char| !c.is_ascii_digit());
    let start = digits.len() - digits.chars().rev().take_while(char::is_ascii_digit).count();
    digits[start..].parse().ok()
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

fn read_model() -> String {
    sysfs::read("/proc/cpuinfo")
        .and_then(|content| {
            content
                .lines()
                .find(|l| l.starts_with("model name"))?
                .split_once(':')
                .map(|(_, model)| model.trim().to_string())
        })
        .unwrap_or_else(|| "Processor".to_string())
}

/// Reads the highest online CPU index from `0-N` in `cpu/online`.
fn online_cpus() -> usize {
    sysfs::read("/sys/devices/system/cpu/online")
        .and_then(|s| s.split_once('-')?.1.parse::<usize>().ok())
        .map_or(1, |last| last + 1)
}

fn read_core_freq(core: usize) -> Option<u32> {
    let path = format!("/sys/devices/system/cpu/cpu{core}/cpufreq/scaling_cur_freq");
    sysfs::parse::<u32>(path).map(|khz| khz / 1000)
}

/// Counts processes (numeric `/proc` entries) and threads (from `/proc/loadavg`).
fn read_process_counts() -> (usize, usize) {
    let processes = fs::read_dir("/proc").map_or(0, |entries| {
        entries
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().parse::<u32>().is_ok())
            .count()
    });

    // The fourth field is `running/total`.
    let threads = sysfs::read("/proc/loadavg")
        .and_then(|c| c.split_whitespace().nth(3)?.split_once('/')?.1.parse().ok())
        .unwrap_or(0);

    (processes, threads)
}

/// System-wide open file descriptors, the closest Linux analogue to handles.
fn read_open_handles() -> usize {
    sysfs::read("/proc/sys/fs/file-nr")
        .and_then(|c| c.split_whitespace().next()?.parse().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_core_labels_numerically() {
        let mut labels = ["Core 10", "Core 2", "Core 1", "Tccd9", "Package id 0", "Unlabelled"];
        labels.sort_by_key(|l| (trailing_number(l), *l));

        assert_eq!(
            labels,
            ["Unlabelled", "Package id 0", "Core 1", "Core 2", "Tccd9", "Core 10"]
        );
    }

    #[test]
    fn parses_cpu_stat_fields() {
        // user nice system idle iowait irq softirq steal
        let times = CpuTimes::parse("100 10 50 800 20 5 5 0".split_whitespace()).unwrap();

        assert_eq!(times.user, 110);
        assert_eq!(times.system, 60);
        assert_eq!(times.idle, 820);
        assert_eq!(times.total(), 990);
    }

    #[test]
    fn rejects_truncated_cpu_stat_lines() {
        assert!(CpuTimes::parse("1 2 3".split_whitespace()).is_none());
    }

    #[test]
    fn usage_is_the_non_idle_share_of_the_interval() {
        let prev = CpuTimes { user: 0, system: 0, idle: 0 };
        let now = CpuTimes { user: 20, system: 5, idle: 75 };
        let usage = now.usage_since(prev);

        assert_eq!(usage.overall, 25.0);
        assert_eq!(usage.user, 20.0);
        assert_eq!(usage.system, 5.0);
        assert_eq!(usage.idle, 75.0);
    }

    #[test]
    fn identical_samples_report_no_usage() {
        let times = CpuTimes { user: 10, system: 10, idle: 10 };
        assert_eq!(times.usage_since(times).idle, 100.0);
    }
}
