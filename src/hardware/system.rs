//! Uptime, load average, network throughput and memory usage.

use super::sysfs::{self, RateMeter};
use super::types::{MemoryMetrics, SystemInfo};

/// `/proc/net/dev` gives each interface 16 counters. We want the first
/// (received bytes) and the ninth (transmitted bytes).
const NET_RECEIVED_BYTES: usize = 0;
const NET_TRANSMITTED_BYTES: usize = 8;

/// Holds the network counters between refreshes so throughput can be measured.
#[derive(Default)]
pub struct SystemCollector {
    received: RateMeter,
    transmitted: RateMeter,
}

impl SystemCollector {
    pub fn collect(&mut self) -> (SystemInfo, MemoryMetrics) {
        let (received_bytes, transmitted_bytes) = read_network_totals();

        let info = SystemInfo {
            uptime_seconds: read_uptime(),
            load_avg: read_load_average(),
            net_rx_kbs: self.received.kb_per_second(received_bytes),
            net_tx_kbs: self.transmitted.kb_per_second(transmitted_bytes),
        };

        (info, read_memory())
    }
}

/// Seconds since boot, from the first field of `/proc/uptime`.
fn read_uptime() -> u64 {
    let Some(contents) = sysfs::read("/proc/uptime") else {
        return 0;
    };
    let Some(seconds) = contents.split_whitespace().next() else {
        return 0;
    };

    // The file stores a fractional value, e.g. "16711.42 128033.79".
    seconds.parse::<f64>().unwrap_or(0.0) as u64
}

/// The 1, 5 and 15 minute load averages, the first three fields of
/// `/proc/loadavg`.
fn read_load_average() -> [f32; 3] {
    let mut averages = [0.0; 3];

    if let Some(contents) = sysfs::read("/proc/loadavg") {
        for (average, field) in averages.iter_mut().zip(contents.split_whitespace()) {
            *average = field.parse().unwrap_or(0.0);
        }
    }

    averages
}

/// Totals bytes received and transmitted across every interface but loopback.
///
/// Each line of `/proc/net/dev` looks like:
///
/// ```text
///   enp5s0: 1234567  8901    0    0    0     0          0         0  7654321 ...
///           ^ received bytes                                         ^ transmitted bytes
/// ```
fn read_network_totals() -> (u64, u64) {
    let Some(contents) = sysfs::read("/proc/net/dev") else {
        return (0, 0);
    };

    let mut received = 0;
    let mut transmitted = 0;

    // The first two lines are column headings.
    for line in contents.lines().skip(2) {
        let Some((interface, counters)) = line.split_once(':') else {
            continue;
        };

        // Loopback traffic never leaves the machine, so it isn't throughput.
        if interface.trim() == "lo" {
            continue;
        }

        let counters: Vec<&str> = counters.split_whitespace().collect();
        received += parse_counter(&counters, NET_RECEIVED_BYTES);
        transmitted += parse_counter(&counters, NET_TRANSMITTED_BYTES);
    }

    (received, transmitted)
}

fn parse_counter(counters: &[&str], index: usize) -> u64 {
    counters.get(index).and_then(|value| value.parse().ok()).unwrap_or(0)
}

/// Reads the `/proc/meminfo` fields the UI shows.
///
/// The file is one `Key: value kB` per line, of which we want seven. Every
/// value is in kibibytes.
fn read_memory() -> MemoryMetrics {
    let Some(contents) = sysfs::read("/proc/meminfo") else {
        return MemoryMetrics::default();
    };

    let mut total = 0;
    let mut available = 0;
    let mut committed = 0;
    let mut cached = 0;
    let mut buffers = 0;
    let mut swap_total = 0;
    let mut swap_free = 0;

    for line in contents.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };

        // Strip the trailing " kB" by taking just the number.
        let kibibytes: u64 = match value.split_whitespace().next() {
            Some(number) => number.parse().unwrap_or(0),
            None => continue,
        };

        match key {
            "MemTotal" => total = kibibytes,
            // What is free for a new process to use, including reclaimable
            // cache — a truer figure than MemFree.
            "MemAvailable" => available = kibibytes,
            "Committed_AS" => committed = kibibytes,
            "Cached" => cached = kibibytes,
            "Buffers" => buffers = kibibytes,
            "SwapTotal" => swap_total = kibibytes,
            "SwapFree" => swap_free = kibibytes,
            _ => {}
        }
    }

    let used = total.saturating_sub(available);
    let percent = if total > 0 { (used as f32 / total as f32) * 100.0 } else { 0.0 };
    let to_bytes = |kibibytes: u64| kibibytes * 1024;

    MemoryMetrics {
        total_bytes: to_bytes(total),
        used_bytes: to_bytes(used),
        available_bytes: to_bytes(available),
        committed_bytes: to_bytes(committed),
        // Page cache and block buffers are both reclaimable, and the UI has
        // room for one figure rather than two.
        cached_bytes: to_bytes(cached + buffers),
        swap_total_bytes: to_bytes(swap_total),
        swap_used_bytes: to_bytes(swap_total.saturating_sub(swap_free)),
        swap_avail_bytes: to_bytes(swap_free),
        percent: percent.clamp(0.0, 100.0),
    }
}
