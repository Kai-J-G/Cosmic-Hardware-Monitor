use std::collections::HashMap;
use std::ffi::CString;
use std::fs;
use std::mem::MaybeUninit;
use std::time::Instant;

use super::types::{MemoryMetrics, ProcessInfo, SystemInfo};

#[derive(Debug)]
pub struct SystemCollector {
    last_net_rx: u64,
    last_net_tx: u64,
    last_net_time: Option<Instant>,
    last_procs: HashMap<u32, (String, u64)>, // pid -> (name, total_ticks)
    last_proc_time: Option<Instant>,
}

impl Default for SystemCollector {
    fn default() -> Self {
        Self {
            last_net_rx: 0,
            last_net_tx: 0,
            last_net_time: None,
            last_procs: HashMap::new(),
            last_proc_time: None,
        }
    }
}

impl SystemCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn collect(&mut self) -> (SystemInfo, MemoryMetrics) {
        let hostname = read_hostname();
        let os_name = read_os_name();
        let kernel = fs::read_to_string("/proc/sys/kernel/osrelease")
            .unwrap_or_else(|_| "Linux".to_string())
            .trim()
            .to_string();

        let uptime_seconds = read_uptime();
        let load_avg = read_load_avg();
        let (mem_used_bytes, mem_total_bytes, mem_percent) = read_memory();
        let memory_metrics = read_memory_metrics();
        let (disk_used_bytes, disk_total_bytes, disk_percent) = read_disk_usage();
        let (net_rx_kbs, net_tx_kbs) = self.read_network_rates();
        let top_processes = self.read_top_processes();
        let spd_temps = read_spd_temps();

        (
            SystemInfo {
                hostname,
                os_name,
                kernel,
                uptime_seconds,
                load_avg,
                mem_used_bytes,
                mem_total_bytes,
                mem_percent,
                spd_temps,
                disk_used_bytes,
                disk_total_bytes,
                disk_percent,
                net_rx_kbs,
                net_tx_kbs,
                top_processes,
            },
            memory_metrics,
        )
    }

    fn read_network_rates(&mut self) -> (f32, f32) {
        let (rx, tx) = read_net_bytes();
        let now = Instant::now();
        let mut rates = (0.0f32, 0.0f32);

        if let Some(last_time) = self.last_net_time {
            let dt = (now - last_time).as_secs_f32();
            if dt > 0.05 {
                let rx_diff = rx.saturating_sub(self.last_net_rx) as f32;
                let tx_diff = tx.saturating_sub(self.last_net_tx) as f32;
                // Rate in KB/s
                rates = ((rx_diff / dt) / 1024.0, (tx_diff / dt) / 1024.0);
            }
        }

        self.last_net_rx = rx;
        self.last_net_tx = tx;
        self.last_net_time = Some(now);
        rates
    }

    fn read_top_processes(&mut self) -> Vec<ProcessInfo> {
        let current_procs = scan_proc_ticks();
        let now = Instant::now();
        let mut result = Vec::new();

        if let Some(last_time) = self.last_proc_time {
            let dt = (now - last_time).as_secs_f32();
            if dt > 0.1 {
                let clk_tck = 100.0f32; // Standard Linux x86_64 CLK_TCK
                let mut name_usages: HashMap<String, f32> = HashMap::new();

                for (pid, (name, ticks)) in &current_procs {
                    if let Some((_, prev_ticks)) = self.last_procs.get(pid) {
                        let dticks = ticks.saturating_sub(*prev_ticks);
                        if dticks > 0 {
                            let pct = ((dticks as f32 / dt) / clk_tck) * 100.0;
                            *name_usages.entry(name.clone()).or_insert(0.0) += pct;
                        }
                    }
                }

                let mut sorted: Vec<(String, f32)> = name_usages.into_iter().collect();
                sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

                for (name, cpu_percent) in sorted.into_iter().take(4) {
                    if cpu_percent > 0.05 {
                        result.push(ProcessInfo {
                            name,
                            cpu_percent: (cpu_percent * 10.0).round() / 10.0,
                        });
                    }
                }
            }
        }

        self.last_procs = current_procs;
        self.last_proc_time = Some(now);
        result
    }
}

#[allow(dead_code)]
pub fn read_system_info() -> SystemInfo {
    SystemCollector::new().collect().0
}

fn read_hostname() -> String {
    if let Ok(h) = fs::read_to_string("/proc/sys/kernel/hostname") {
        let trimmed = h.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    if let Ok(h) = fs::read_to_string("/etc/hostname") {
        let trimmed = h.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    "Linux".to_string()
}

fn read_os_name() -> String {
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                let val = line["PRETTY_NAME=".len()..].trim_matches('"');
                return val.to_string();
            }
        }
    }
    "Linux".to_string()
}

fn read_uptime() -> u64 {
    if let Ok(content) = fs::read_to_string("/proc/uptime") {
        if let Some(first) = content.split_whitespace().next() {
            if let Ok(seconds) = first.parse::<f64>() {
                return seconds as u64;
            }
        }
    }
    0
}

fn read_load_avg() -> [f32; 3] {
    let mut loads = [0.0, 0.0, 0.0];
    if let Ok(content) = fs::read_to_string("/proc/loadavg") {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 3 {
            loads[0] = parts[0].parse().unwrap_or(0.0);
            loads[1] = parts[1].parse().unwrap_or(0.0);
            loads[2] = parts[2].parse().unwrap_or(0.0);
        }
    }
    loads
}

fn read_memory() -> (u64, u64, f32) {
    let mut total_kb = 0u64;
    let mut avail_kb = 0u64;

    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                total_kb = parse_meminfo_kb(line);
            } else if line.starts_with("MemAvailable:") {
                avail_kb = parse_meminfo_kb(line);
            }
        }
    }

    if total_kb == 0 {
        return (0, 0, 0.0);
    }

    let used_kb = total_kb.saturating_sub(avail_kb);
    let total_bytes = total_kb * 1024;
    let used_bytes = used_kb * 1024;
    let percent = (used_kb as f32 / total_kb as f32) * 100.0;

    (used_bytes, total_bytes, percent.clamp(0.0, 100.0))
}

fn parse_meminfo_kb(line: &str) -> u64 {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        parts[1].parse().unwrap_or(0)
    } else {
        0
    }
}

pub fn read_memory_metrics() -> MemoryMetrics {
    let mut total_kb = 0u64;
    let mut avail_kb = 0u64;
    let mut committed_kb = 0u64;
    let mut cached_kb = 0u64;
    let mut buffers_kb = 0u64;
    let mut swap_total_kb = 0u64;
    let mut swap_free_kb = 0u64;

    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                total_kb = parse_meminfo_kb(line);
            } else if line.starts_with("MemAvailable:") {
                avail_kb = parse_meminfo_kb(line);
            } else if line.starts_with("Committed_AS:") {
                committed_kb = parse_meminfo_kb(line);
            } else if line.starts_with("Cached:") {
                cached_kb = parse_meminfo_kb(line);
            } else if line.starts_with("Buffers:") {
                buffers_kb = parse_meminfo_kb(line);
            } else if line.starts_with("SwapTotal:") {
                swap_total_kb = parse_meminfo_kb(line);
            } else if line.starts_with("SwapFree:") {
                swap_free_kb = parse_meminfo_kb(line);
            }
        }
    }

    let used_kb = total_kb.saturating_sub(avail_kb);
    let swap_used_kb = swap_total_kb.saturating_sub(swap_free_kb);
    let percent = if total_kb > 0 {
        (used_kb as f32 / total_kb as f32) * 100.0
    } else {
        0.0
    };

    MemoryMetrics {
        total_bytes: total_kb * 1024,
        used_bytes: used_kb * 1024,
        available_bytes: avail_kb * 1024,
        committed_bytes: committed_kb * 1024,
        cached_bytes: (cached_kb + buffers_kb) * 1024,
        swap_total_bytes: swap_total_kb * 1024,
        swap_used_bytes: swap_used_kb * 1024,
        swap_avail_bytes: swap_free_kb * 1024,
        percent: percent.clamp(0.0, 100.0),
    }
}

fn read_disk_usage() -> (u64, u64, f32) {
    let path = match CString::new("/") {
        Ok(p) => p,
        Err(_) => return (0, 0, 0.0),
    };
    let mut stat = MaybeUninit::<libc::statvfs>::uninit();
    let res = unsafe { libc::statvfs(path.as_ptr(), stat.as_mut_ptr()) };
    if res == 0 {
        let stat = unsafe { stat.assume_init() };
        let block_size = if stat.f_frsize > 0 {
            stat.f_frsize
        } else {
            stat.f_bsize
        } as u64;
        let total_bytes = stat.f_blocks as u64 * block_size;
        let free_bytes = stat.f_bavail as u64 * block_size;
        let used_bytes = total_bytes.saturating_sub(free_bytes);
        let percent = if total_bytes > 0 {
            (used_bytes as f32 / total_bytes as f32) * 100.0
        } else {
            0.0
        };
        (used_bytes, total_bytes, percent.clamp(0.0, 100.0))
    } else {
        (0, 0, 0.0)
    }
}

fn read_net_bytes() -> (u64, u64) {
    let mut total_rx = 0u64;
    let mut total_tx = 0u64;
    if let Ok(content) = fs::read_to_string("/proc/net/dev") {
        for line in content.lines().skip(2) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            let iface = parts[0].trim_end_matches(':');
            if iface == "lo" {
                continue;
            }
            let (rx_str, tx_idx) = if parts[0].ends_with(':') {
                (parts.get(1).copied(), 9)
            } else if let Some(idx) = parts[0].find(':') {
                (Some(&parts[0][idx + 1..]), 8)
            } else {
                (parts.get(1).copied(), 9)
            };
            if let Some(rx_s) = rx_str {
                if let Ok(rx) = rx_s.parse::<u64>() {
                    total_rx += rx;
                }
            }
            if let Some(tx_s) = parts.get(tx_idx) {
                if let Ok(tx) = tx_s.parse::<u64>() {
                    total_tx += tx;
                }
            }
        }
    }
    (total_rx, total_tx)
}

fn scan_proc_ticks() -> HashMap<u32, (String, u64)> {
    let mut map = HashMap::new();
    let Ok(entries) = fs::read_dir("/proc") else {
        return map;
    };

    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let Ok(pid) = name.to_string_lossy().parse::<u32>() else {
            continue;
        };

        let stat_path = entry.path().join("stat");
        if let Ok(content) = fs::read_to_string(stat_path) {
            if let (Some(l), Some(r)) = (content.find('('), content.rfind(')')) {
                if l < r && r + 2 < content.len() {
                    let comm = content[l + 1..r].to_string();
                    let after = &content[r + 2..];
                    let parts: Vec<&str> = after.split_whitespace().collect();
                    if parts.len() >= 13 {
                        let utime: u64 = parts[11].parse().unwrap_or(0);
                        let stime: u64 = parts[12].parse().unwrap_or(0);
                        map.insert(pid, (comm, utime + stime));
                    }
                }
            }
        }
    }
    map
}

fn read_spd_temps() -> Vec<(String, f32)> {
    let mut list = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/hwmon") else {
        return list;
    };

    let mut dimm_idx = 1;
    for entry in entries.flatten() {
        let path = entry.path();
        if let Ok(name) = fs::read_to_string(path.join("name")) {
            let name = name.trim();
            if name.starts_with("spd5118") || name.starts_with("ee1004") {
                if let Ok(content) = fs::read_to_string(path.join("temp1_input")) {
                    if let Ok(val) = content.trim().parse::<f32>() {
                        list.push((format!("DIMM {dimm_idx} ({name})"), val / 1000.0));
                        dimm_idx += 1;
                    }
                }
            }
        }
    }
    list
}
