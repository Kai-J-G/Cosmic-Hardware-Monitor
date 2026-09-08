//! Uptime, load average, network throughput and memory usage.

use super::sysfs::{self, RateMeter};
use super::types::{MemoryMetrics, SystemInfo};

#[derive(Default)]
pub struct SystemCollector {
    net: RateMeter,
}

impl SystemCollector {
    pub fn collect(&mut self) -> (SystemInfo, MemoryMetrics) {
        let (rx, tx) = read_net_bytes();
        let (net_rx_kbs, net_tx_kbs) = self.net.sample(rx, tx);

        let info = SystemInfo {
            uptime_seconds: read_uptime(),
            load_avg: read_load_avg(),
            net_rx_kbs,
            net_tx_kbs,
        };

        (info, read_memory())
    }
}

fn read_uptime() -> u64 {
    sysfs::read("/proc/uptime")
        .and_then(|c| c.split_whitespace().next()?.parse::<f64>().ok())
        .unwrap_or(0.0) as u64
}

/// The 1, 5 and 15 minute load averages.
fn read_load_avg() -> [f32; 3] {
    let mut loads = [0.0; 3];
    if let Some(content) = sysfs::read("/proc/loadavg") {
        for (slot, field) in loads.iter_mut().zip(content.split_whitespace()) {
            *slot = field.parse().unwrap_or(0.0);
        }
    }
    loads
}

/// Sums received and transmitted bytes across every interface but loopback.
fn read_net_bytes() -> (u64, u64) {
    let Some(content) = sysfs::read("/proc/net/dev") else {
        return (0, 0);
    };

    content
        .lines()
        // Two header rows, then `iface: rx_bytes ... tx_bytes ...` per device.
        .skip(2)
        .filter_map(|line| {
            let (iface, counters) = line.split_once(':')?;
            if iface.trim() == "lo" {
                return None;
            }
            let fields: Vec<&str> = counters.split_whitespace().collect();
            Some((fields.first()?.parse().ok()?, fields.get(8)?.parse().ok()?))
        })
        .fold((0u64, 0u64), |(rx, tx), (drx, dtx): (u64, u64)| (rx + drx, tx + dtx))
}

/// Parses the handful of `/proc/meminfo` fields the UI shows.
///
/// Every value there is in kibibytes.
fn read_memory() -> MemoryMetrics {
    let Some(content) = sysfs::read("/proc/meminfo") else {
        return MemoryMetrics::default();
    };

    let mut fields = [
        ("MemTotal:", 0u64),
        ("MemAvailable:", 0),
        ("Committed_AS:", 0),
        ("Cached:", 0),
        ("Buffers:", 0),
        ("SwapTotal:", 0),
        ("SwapFree:", 0),
    ];

    for line in content.lines() {
        if let Some((_, value)) = fields.iter_mut().find(|(key, _)| line.starts_with(key)) {
            *value = line.split_whitespace().nth(1).and_then(|v| v.parse().ok()).unwrap_or(0);
        }
    }

    let [total, available, committed, cached, buffers, swap_total, swap_free] =
        fields.map(|(_, kib)| kib * 1024);

    let used = total.saturating_sub(available);
    let percent = if total > 0 { (used as f32 / total as f32) * 100.0 } else { 0.0 };

    MemoryMetrics {
        total_bytes: total,
        used_bytes: used,
        available_bytes: available,
        committed_bytes: committed,
        // Page cache and block buffers are both reclaimable; the UI shows one figure.
        cached_bytes: cached + buffers,
        swap_total_bytes: swap_total,
        swap_used_bytes: swap_total.saturating_sub(swap_free),
        swap_avail_bytes: swap_free,
        percent: percent.clamp(0.0, 100.0),
    }
}
