//! Drive temperatures, disk throughput and mounted-partition usage.

use std::collections::HashSet;
use std::ffi::CString;
use std::mem::MaybeUninit;

use super::sandbox;
use super::sysfs::{self, Hwmon, RateMeter};
use super::types::{PartitionInfo, StorageInfo, StorageMetrics};

/// `/proc/diskstats` counts I/O in 512-byte sectors whatever the drive's real
/// sector size, so this constant is fixed rather than hardware-dependent.
const SECTOR_BYTES: u64 = 512;

/// `/proc/diskstats` columns: the device name, then the sector counts.
const DISKSTATS_DEVICE_NAME: usize = 2;
const DISKSTATS_SECTORS_READ: usize = 5;
const DISKSTATS_SECTORS_WRITTEN: usize = 9;

/// hwmon drivers that report drive temperatures. `nvme` covers NVMe SSDs;
/// `drivetemp` covers SATA drives, and is not loaded by default on all
/// distributions.
const DRIVE_CHIPS: [&str; 2] = ["nvme", "drivetemp"];

/// Mount points not worth listing: container and package-manager mounts that
/// duplicate a filesystem already shown, and system directories that are
/// usually just the root filesystem again.
const IGNORED_MOUNTS: [&str; 4] = ["/var/", "/snap", "/root", "/srv"];

/// Holds the disk counters between refreshes so throughput can be measured.
#[derive(Default)]
pub struct StorageCollector {
    reads: RateMeter,
    writes: RateMeter,
}

impl StorageCollector {
    pub fn collect(&mut self, chips: &[Hwmon]) -> (Vec<StorageInfo>, StorageMetrics) {
        let (sectors_read, sectors_written) = read_disk_totals();

        let metrics = StorageMetrics {
            read_kbs: self.reads.kb_per_second(sectors_read * SECTOR_BYTES),
            write_kbs: self.writes.kb_per_second(sectors_written * SECTOR_BYTES),
            partitions: read_partitions(),
        };

        (read_drive_temps(chips), metrics)
    }
}

/// Reads each drive's composite temperature.
fn read_drive_temps(chips: &[Hwmon]) -> Vec<StorageInfo> {
    let mut drives = Vec::new();

    for chip in chips {
        let is_drive = DRIVE_CHIPS.iter().any(|driver| chip.name.starts_with(driver));
        if !is_drive {
            continue;
        }

        // A drive with no readable temperature is not worth a row.
        if let Some(temp) = chip.temp(1) {
            drives.push(StorageInfo { name: drive_model(chip), temp });
        }
    }

    drives
}

/// Names a drive by its advertised model, falling back to the driver name.
fn drive_model(chip: &Hwmon) -> String {
    match sysfs::read(chip.dir.join("device/model")) {
        Some(model) if !model.is_empty() => format!("NVMe {model}"),
        _ => format!("Storage ({})", chip.name),
    }
}

/// Totals sectors read and written across whole disks.
///
/// Partitions are skipped: the kernel counts their I/O against both the
/// partition and its parent disk, so including them would double the figure.
fn read_disk_totals() -> (u64, u64) {
    let Some(contents) = sysfs::read("/proc/diskstats") else {
        return (0, 0);
    };

    let mut sectors_read = 0;
    let mut sectors_written = 0;

    for line in contents.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();

        let Some(device) = fields.get(DISKSTATS_DEVICE_NAME) else {
            continue;
        };
        if !is_whole_disk(device) {
            continue;
        }

        sectors_read += parse_field(&fields, DISKSTATS_SECTORS_READ);
        sectors_written += parse_field(&fields, DISKSTATS_SECTORS_WRITTEN);
    }

    (sectors_read, sectors_written)
}

/// Distinguishes a disk from one of its partitions.
///
/// `nvme0n1` and `sda` are whole disks; `nvme0n1p2` and `sda1` are partitions.
fn is_whole_disk(device: &str) -> bool {
    let nvme_disk = device.starts_with("nvme") && !device.contains('p');
    // "sda" is a disk, "sda1" is not — hence the exact length.
    let sata_disk = device.starts_with("sd") && device.len() == 3;

    nvme_disk || sata_disk
}

fn parse_field(fields: &[&str], index: usize) -> u64 {
    fields.get(index).and_then(|value| value.parse().ok()).unwrap_or(0)
}

/// Lists the mounted filesystems worth showing, root first.
///
/// Returns nothing inside a Flatpak sandbox, where `/proc/mounts` describes the
/// sandbox rather than the machine — see [`super::sandbox`].
fn read_partitions() -> Vec<PartitionInfo> {
    if sandbox::is_flatpak() {
        return Vec::new();
    }

    let Some(contents) = sysfs::read("/proc/mounts") else {
        return Vec::new();
    };

    let mut partitions = Vec::new();
    let mut seen_mounts = HashSet::new();

    // Each line is `device mount filesystem options ...`.
    for line in contents.lines() {
        let mut fields = line.split_whitespace();
        let (Some(device), Some(mount), Some(filesystem)) =
            (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };

        // Anything not backed by a block device is a pseudo-filesystem.
        if !device.starts_with("/dev/") {
            continue;
        }
        if IGNORED_MOUNTS.iter().any(|ignored| mount.starts_with(ignored) || mount.contains(ignored))
        {
            continue;
        }
        // A filesystem can be mounted more than once; show it once.
        if !seen_mounts.insert(mount.to_string()) {
            continue;
        }

        let Some((total_bytes, free_bytes)) = disk_usage(mount) else {
            continue;
        };
        let used_bytes = total_bytes.saturating_sub(free_bytes);

        partitions.push(PartitionInfo {
            mount: mount.to_string(),
            filesystem: filesystem.to_string(),
            total_bytes,
            free_bytes,
            used_bytes,
            percent_used: ((used_bytes as f32 / total_bytes as f32) * 100.0).clamp(0.0, 100.0),
        });
    }

    // Root first, then the rest alphabetically. Comparing `mount != "/"` sorts
    // false (root) ahead of true (everything else).
    partitions.sort_by(|a, b| (a.mount != "/", &a.mount).cmp(&(b.mount != "/", &b.mount)));
    partitions
}

/// Total and available bytes for a mount point.
///
/// Returns `None` for a filesystem reporting zero size, which is how
/// pseudo-filesystems appear.
fn disk_usage(mount: &str) -> Option<(u64, u64)> {
    let path = CString::new(mount).ok()?;
    let mut stat = MaybeUninit::<libc::statvfs>::uninit();

    // SAFETY: `path` is a valid NUL-terminated string, and `stat` is only read
    // after statvfs reports success, which means the kernel filled it in.
    let stat = unsafe {
        if libc::statvfs(path.as_ptr(), stat.as_mut_ptr()) != 0 {
            return None;
        }
        stat.assume_init()
    };

    // `f_frsize` is the real block size; `f_bsize` is a hint some filesystems
    // set instead.
    let block_size = if stat.f_frsize > 0 { stat.f_frsize } else { stat.f_bsize } as u64;
    let total = stat.f_blocks as u64 * block_size;
    // `f_bavail` excludes root-reserved blocks, so it matches what a normal
    // user can actually fill.
    let free = stat.f_bavail as u64 * block_size;

    if total == 0 {
        return None;
    }

    Some((total, free))
}
