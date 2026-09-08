//! Formatting helpers shared by the views, so the same quantity always reads
//! the same way wherever it appears.
//!
//! Unit names come from the translation files, so a translator can adjust
//! spacing or wording without touching code.

use crate::fl;

/// Byte counts as MB below a gibibyte, GB above it.
pub fn bytes(bytes: u64) -> String {
    let mb = bytes as f64 / (1024.0 * 1024.0);

    // Bound to locals because Fluent arguments borrow rather than own.
    if mb >= 1024.0 {
        let value = format!("{:.2}", mb / 1024.0);
        fl!("unit-gigabytes", value = value.as_str())
    } else {
        let value = format!("{mb:.0}");
        fl!("unit-megabytes", value = value.as_str())
    }
}

/// Gibibytes, for the summary lines that always speak in GB.
pub fn gb(bytes: u64) -> f64 {
    bytes as f64 / 1_073_741_824.0
}

/// Throughput given in KB/s, promoted to MB/s once it gets large.
pub fn rate(kbs: f32) -> String {
    if kbs >= 1024.0 {
        let value = format!("{:.1}", kbs / 1024.0);
        fl!("unit-mb-per-second", value = value.as_str())
    } else {
        let value = format!("{kbs:.1}");
        fl!("unit-kb-per-second", value = value.as_str())
    }
}

/// A percentage, e.g. `42%`.
pub fn percent(value: f32, decimals: usize) -> String {
    let value = format!("{value:.*}", decimals);
    fl!("unit-percent", value = value.as_str())
}

/// Uptime at the coarsest granularity that still says something useful.
pub fn uptime(seconds: u64) -> String {
    let (days, hours, minutes) = (seconds / 86400, (seconds % 86400) / 3600, (seconds % 3600) / 60);
    match (days, hours) {
        (0, 0) => fl!("uptime-minutes", minutes = minutes),
        (0, _) => fl!("uptime-hours", hours = hours, minutes = minutes),
        _ => fl!("uptime-days", days = days, hours = hours, minutes = minutes),
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
