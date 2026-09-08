//! Detects whether we are running inside a Flatpak sandbox.
//!
//! Most of what this applet reads works the same either way: `/proc/stat`,
//! `/proc/meminfo` and `/proc/diskstats` are system-wide, and hwmon sensors are
//! readable given the right filesystem permissions. Two things are not:
//!
//! - **Process count.** The sandbox has its own PID namespace, so `/proc` lists
//!   only the handful of processes inside it rather than the machine's.
//! - **Mounted partitions.** The sandbox's `/proc/mounts` describes the sandbox,
//!   and is full of bind mounts of the same device at paths like
//!   `/usr/share/runtime/locale`.
//!
//! Neither can be recovered without granting the sandbox broad access to the
//! host filesystem, which Flathub discourages and which would not fix the
//! process count anyway. So those two readings are simply omitted, rather than
//! shown wrong.

use std::path::Path;
use std::sync::OnceLock;

/// Whether this process is confined by Flatpak.
///
/// Flatpak places this file in every sandbox it builds; it is the documented
/// way for an application to know it is running inside one.
pub fn is_flatpak() -> bool {
    static IS_FLATPAK: OnceLock<bool> = OnceLock::new();

    *IS_FLATPAK.get_or_init(|| Path::new("/.flatpak-info").exists())
}
