use std::collections::HashMap;
use std::fs;
use std::time::Instant;

use super::types::{CpuCoreInfo, CpuInfo};

#[derive(Default, Clone, Copy)]
struct CpuTimes {
    user: u64,
    system: u64,
    idle: u64,
    total: u64,
}

#[derive(Default, Clone)]
pub struct CpuStatHistory {
    prev_overall: Option<CpuTimes>,
    prev_cores: HashMap<usize, (u64, u64)>,
    prev_energy_uj: Option<u64>,
    prev_energy_time: Option<Instant>,
}

impl CpuStatHistory {
    pub fn read_usage(&mut self) -> (f32, f32, f32, f32, HashMap<usize, f32>) {
        let content = match fs::read_to_string("/proc/stat") {
            Ok(c) => c,
            Err(_) => return (0.0, 0.0, 0.0, 100.0, HashMap::new()),
        };

        let mut overall_usage = 0.0;
        let mut user_usage = 0.0;
        let mut sys_usage = 0.0;
        let mut idle_usage = 100.0;
        let mut core_usages = HashMap::new();

        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            if parts[0] == "cpu" {
                let user: u64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                let nice: u64 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
                let system: u64 = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);
                let idle: u64 = parts.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
                let iowait: u64 = parts.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
                let irq: u64 = parts.get(6).and_then(|s| s.parse().ok()).unwrap_or(0);
                let softirq: u64 = parts.get(7).and_then(|s| s.parse().ok()).unwrap_or(0);
                let steal: u64 = parts.get(8).and_then(|s| s.parse().ok()).unwrap_or(0);

                let idle_all = idle + iowait;
                let user_all = user + nice;
                let sys_all = system + irq + softirq + steal;
                let total = user_all + sys_all + idle_all;

                let cur = CpuTimes {
                    user: user_all,
                    system: sys_all,
                    idle: idle_all,
                    total,
                };
                if let Some(prev) = self.prev_overall {
                    let diff_total = total.saturating_sub(prev.total);
                    if diff_total > 0 {
                        let diff_user = user_all.saturating_sub(prev.user);
                        let diff_sys = sys_all.saturating_sub(prev.system);
                        let diff_idle = idle_all.saturating_sub(prev.idle);
                        user_usage = (diff_user as f32 / diff_total as f32) * 100.0;
                        sys_usage = (diff_sys as f32 / diff_total as f32) * 100.0;
                        idle_usage = (diff_idle as f32 / diff_total as f32) * 100.0;
                        overall_usage = (1.0 - (diff_idle as f32 / diff_total as f32)) * 100.0;
                    }
                }
                self.prev_overall = Some(cur);
            } else if parts[0].starts_with("cpu") && parts[0][3..].chars().all(|c| c.is_ascii_digit()) {
                if let Ok(core_idx) = parts[0][3..].parse::<usize>() {
                    if let Some((idle, total)) = parse_cpu_stat_line(&parts[1..]) {
                        if let Some(&(prev_idle, prev_total)) = self.prev_cores.get(&core_idx) {
                            let diff_total = total.saturating_sub(prev_total);
                            let diff_idle = idle.saturating_sub(prev_idle);
                            if diff_total > 0 {
                                let usage = (1.0 - (diff_idle as f32 / diff_total as f32)) * 100.0;
                                core_usages.insert(core_idx, usage.clamp(0.0, 100.0));
                            }
                        }
                        self.prev_cores.insert(core_idx, (idle, total));
                    }
                }
            }
        }

        (
            overall_usage.clamp(0.0, 100.0),
            user_usage.clamp(0.0, 100.0),
            sys_usage.clamp(0.0, 100.0),
            idle_usage.clamp(0.0, 100.0),
            core_usages,
        )
    }
}

fn parse_cpu_stat_line(fields: &[&str]) -> Option<(u64, u64)> {
    if fields.len() < 4 {
        return None;
    }
    let user: u64 = fields.get(0)?.parse().ok()?;
    let nice: u64 = fields.get(1)?.parse().ok()?;
    let system: u64 = fields.get(2)?.parse().ok()?;
    let idle: u64 = fields.get(3)?.parse().ok()?;
    let iowait: u64 = fields.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
    let irq: u64 = fields.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
    let softirq: u64 = fields.get(6).and_then(|s| s.parse().ok()).unwrap_or(0);
    let steal: u64 = fields.get(7).and_then(|s| s.parse().ok()).unwrap_or(0);

    let idle_all = idle + iowait;
    let total = user + nice + system + idle_all + irq + softirq + steal;
    Some((idle_all, total))
}

pub fn read_cpu_info(stat_history: &mut CpuStatHistory) -> CpuInfo {
    let model = parse_cpu_model();
    let (overall_usage, user_usage, sys_usage, idle_usage, core_usages) = stat_history.read_usage();

    // Enumerate temperature sensors from hwmon
    let mut package_temp = None;
    let mut hwmon_cores = Vec::new();

    if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
        for entry in entries.flatten() {
            let path = entry.path();
            let name_path = path.join("name");
            let Ok(chip_name) = fs::read_to_string(&name_path) else {
                continue;
            };
            let chip_name = chip_name.trim();

            if chip_name == "k10temp" || chip_name == "zenpower" {
                // AMD K10 / Zen
                if let Ok(temp_entries) = fs::read_dir(&path) {
                    for temp_entry in temp_entries.flatten() {
                        let fname = temp_entry.file_name();
                        let fname_str = fname.to_string_lossy();
                        if fname_str.starts_with("temp") && fname_str.ends_with("_input") {
                            let prefix = &fname_str[..fname_str.len() - 6];
                            let label = fs::read_to_string(path.join(format!("{prefix}_label")))
                                .unwrap_or_else(|_| prefix.to_string())
                                .trim()
                                .to_string();

                            if let Ok(val_str) = fs::read_to_string(temp_entry.path()) {
                                if let Ok(val) = val_str.trim().parse::<f32>() {
                                    let temp_c = val / 1000.0;
                                    if label.eq_ignore_ascii_case("tctl")
                                        || label.eq_ignore_ascii_case("tctl/tdie")
                                    {
                                        package_temp = Some(temp_c);
                                    } else if label.to_lowercase().contains("tccd") {
                                        hwmon_cores.push((label, temp_c));
                                    }
                                }
                            }
                        }
                    }
                }
            } else if chip_name == "coretemp" {
                // Intel Core
                if let Ok(temp_entries) = fs::read_dir(&path) {
                    for temp_entry in temp_entries.flatten() {
                        let fname = temp_entry.file_name();
                        let fname_str = fname.to_string_lossy();
                        if fname_str.starts_with("temp") && fname_str.ends_with("_input") {
                            let prefix = &fname_str[..fname_str.len() - 6];
                            let label = fs::read_to_string(path.join(format!("{prefix}_label")))
                                .unwrap_or_else(|_| prefix.to_string())
                                .trim()
                                .to_string();

                            if let Ok(val_str) = fs::read_to_string(temp_entry.path()) {
                                if let Ok(val) = val_str.trim().parse::<f32>() {
                                    let temp_c = val / 1000.0;
                                    if label.to_lowercase().contains("package") {
                                        package_temp = Some(temp_c);
                                    } else if label.to_lowercase().contains("core") {
                                        hwmon_cores.push((label, temp_c));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let pkg_temp = package_temp.unwrap_or(40.0);

    // Build per-core list
    let mut cores = Vec::new();
    let num_cpus = num_online_cpus();

    if !hwmon_cores.is_empty() && hwmon_cores.len() >= num_cpus {
        for (idx, (label, temp)) in hwmon_cores.into_iter().enumerate() {
            let usage = core_usages.get(&idx).copied().unwrap_or(overall_usage);
            let freq = read_core_freq(idx);
            cores.push(CpuCoreInfo {
                id: idx,
                label,
                temp,
                usage_percent: usage,
                freq_mhz: freq,
            });
        }
    } else {
        for idx in 0..num_cpus {
            let usage = core_usages.get(&idx).copied().unwrap_or(overall_usage);
            let freq = read_core_freq(idx);
            let temp = if let Some((_, ccd_temp)) = hwmon_cores.first() {
                *ccd_temp
            } else {
                pkg_temp
            };

            cores.push(CpuCoreInfo {
                id: idx,
                label: format!("Core {}", idx),
                temp,
                usage_percent: usage,
                freq_mhz: freq,
            });
        }
    }

    let freqs: Vec<u32> = cores.iter().filter_map(|c| c.freq_mhz).collect();
    let avg_freq_mhz = if !freqs.is_empty() {
        (freqs.iter().map(|&f| f as u64).sum::<u64>() / freqs.len() as u64) as u32
    } else {
        0
    };

    let power_watts = read_cpu_power(stat_history);
    let (num_processes, num_threads) = read_threads_and_procs();
    let num_handles = read_system_handles();

    CpuInfo {
        model,
        package_temp: pkg_temp,
        cores,
        overall_usage,
        user_usage,
        sys_usage,
        idle_usage,
        avg_freq_mhz,
        power_watts,
        num_processes,
        num_threads,
        num_handles,
    }
}

fn read_cpu_power(history: &mut CpuStatHistory) -> Option<f32> {
    let content = fs::read_to_string("/sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj").ok()?;
    let energy_uj: u64 = content.trim().parse().ok()?;
    let now = Instant::now();
    let mut power_watts = None;

    if let (Some(prev_energy), Some(prev_time)) = (history.prev_energy_uj, history.prev_energy_time) {
        let dt = (now - prev_time).as_secs_f32();
        if dt > 0.05 {
            let diff_uj = energy_uj.saturating_sub(prev_energy);
            let watts = (diff_uj as f32 / 1_000_000.0) / dt;
            if (0.0..=500.0).contains(&watts) {
                power_watts = Some((watts * 10.0).round() / 10.0);
            }
        }
    }

    history.prev_energy_uj = Some(energy_uj);
    history.prev_energy_time = Some(now);
    power_watts
}

fn read_system_handles() -> usize {
    if let Ok(content) = fs::read_to_string("/proc/sys/fs/file-nr") {
        if let Some(first) = content.split_whitespace().next() {
            if let Ok(handles) = first.parse::<usize>() {
                return handles;
            }
        }
    }
    0
}

fn read_threads_and_procs() -> (usize, usize) {
    let mut threads = 0;
    if let Ok(content) = fs::read_to_string("/proc/loadavg") {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 4 {
            if let Some(pos) = parts[3].find('/') {
                threads = parts[3][pos + 1..].parse().unwrap_or(0);
            }
        }
    }

    let mut procs = 0;
    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.chars().all(|c| c.is_ascii_digit()) {
                procs += 1;
            }
        }
    }

    (procs, threads)
}

fn parse_cpu_model() -> String {
    if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
        for line in content.lines() {
            if line.starts_with("model name") {
                if let Some(pos) = line.find(':') {
                    return line[pos + 1..].trim().to_string();
                }
            }
        }
    }
    "Processor".to_string()
}

fn num_online_cpus() -> usize {
    if let Ok(content) = fs::read_to_string("/sys/devices/system/cpu/online") {
        let trimmed = content.trim();
        if let Some(pos) = trimmed.find('-') {
            if let Ok(end) = trimmed[pos + 1..].parse::<usize>() {
                return end + 1;
            }
        }
    }
    1
}

fn read_core_freq(core_idx: usize) -> Option<u32> {
    let path = format!("/sys/devices/system/cpu/cpu{core_idx}/cpufreq/scaling_cur_freq");
    let content = fs::read_to_string(path).ok()?;
    let khz: u32 = content.trim().parse().ok()?;
    Some(khz / 1000)
}
