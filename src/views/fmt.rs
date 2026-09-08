//! Formatting helpers shared by the views, so the same quantity always reads
//! the same way wherever it appears.

/// Byte counts as MB below a gibibyte, GB above it.
pub fn bytes(bytes: u64) -> String {
    let mb = bytes as f64 / (1024.0 * 1024.0);
    if mb >= 1024.0 {
        format!("{:.2} GB", mb / 1024.0)
    } else {
        format!("{mb:.0} MB")
    }
}

/// Gibibytes, for the summary lines that always speak in GB.
pub fn gb(bytes: u64) -> f64 {
    bytes as f64 / 1_073_741_824.0
}

/// Throughput given in KB/s, promoted to MB/s once it gets large.
pub fn rate(kbs: f32) -> String {
    if kbs >= 1024.0 {
        format!("{:.1} MB/s", kbs / 1024.0)
    } else {
        format!("{kbs:.1} KB/s")
    }
}

/// Uptime at the coarsest granularity that still says something useful.
pub fn uptime(seconds: u64) -> String {
    let (days, hours, minutes) = (seconds / 86400, (seconds % 86400) / 3600, (seconds % 3600) / 60);
    match (days, hours) {
        (0, 0) => format!("{minutes} minutes"),
        (0, _) => format!("{hours} hours, {minutes} minutes"),
        _ => format!("{days}d, {hours}h, {minutes}m"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scales_bytes_at_the_gibibyte() {
        assert_eq!(bytes(512 * 1024 * 1024), "512 MB");
        assert_eq!(bytes(2 * 1024 * 1024 * 1024), "2.00 GB");
    }

    #[test]
    fn scales_rates_at_the_mebibyte() {
        assert_eq!(rate(512.0), "512.0 KB/s");
        assert_eq!(rate(2048.0), "2.0 MB/s");
    }

    #[test]
    fn uptime_uses_the_coarsest_useful_unit() {
        assert_eq!(uptime(90), "1 minutes");
        assert_eq!(uptime(3 * 3600 + 120), "3 hours, 2 minutes");
        assert_eq!(uptime(2 * 86400 + 3600), "2d, 1h, 0m");
    }
}
