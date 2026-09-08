//! Drive temperatures, disk throughput and mounted-partition usage.

use std::collections::HashSet;
use std::ffi::CString;
use std::mem::MaybeUninit;

use super::sysfs::{self, Hwmon, RateMeter};
use super::types::{PartitionInfo, StorageInfo, StorageMetrics};

/// A disk sector is 512 bytes in `/proc/diskstats`, regardless of the
/// hardware's real sector size.
const SECTOR_BYTES: u64 = 512;

/// hwmon drivers that expose drive temperatures.
const DRIVE_CHIPS: [&str; 2] = ["nvme", "drivetemp"];

/// Mount points that duplicate or clutter the partition list.
const IGNORED_MOUNTS: [&str; 4] = ["/var/", "/snap", "/root", "/srv"];

#[derive(Default)]
pub struct StorageCollector {
    io: RateMeter,
}

impl StorageCollector {
    pub fn collect(&mut self, chips: &[Hwmon]) -> (Vec<StorageInfo>, StorageMetrics) {
        let (sectors_read, sectors_written) = read_diskstats();
        let (read_kbs, write_kbs) = self
            .io
            .sample(sectors_read * SECTOR_BYTES, sectors_written * SECTOR_BYTES);

        let metrics = StorageMetrics { read_kbs, write_kbs, partitions: read_partitions() };
        (read_drive_temps(chips), metrics)
    }
}

/// Reads each drive's composite temperature (`temp1_input`).
fn read_drive_temps(chips: &[Hwmon]) -> Vec<StorageInfo> {
    chips
        .iter()
        .filter(|c| DRIVE_CHIPS.iter().any(|d| c.name.starts_with(d)))
        .filter_map(|chip| Some(StorageInfo { name: drive_model(chip), temp: chip.temp(1)? }))
        .collect()
}

/// Prefers the drive's advertised model over the bare driver name.
fn drive_model(chip: &Hwmon) -> String {
    sysfs::read(chip.dir.join("device/model"))
        .filter(|m| !m.is_empty())
        .map(|model| format!("NVMe {model}"))
        .unwrap_or_else(|| format!("Storage ({})", chip.name))
}

/// Totals read and written sectors across whole disks.
///
/// Partitions are skipped so their I/O is not counted twice on top of the
/// parent disk's.
fn read_diskstats() -> (u64, u64) {
    let Some(content) = sysfs::read("/proc/diskstats") else {
        return (0, 0);
    };

    content
        .lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            let name = fields.get(2)?;
            if !is_whole_disk(name) {
                return None;
            }
            Some((fields.get(5)?.parse().ok()?, fields.get(9)?.parse().ok()?))
        })
        .fold((0u64, 0u64), |(r, w), (dr, dw): (u64, u64)| (r + dr, w + dw))
}

/// `nvme0n1` and `sda` are disks; `nvme0n1p2` and `sda1` are partitions of one.
fn is_whole_disk(name: &str) -> bool {
    (name.starts_with("nvme") && !name.contains('p')) || (name.starts_with("sd") && name.len() == 3)
}

/// Lists real, distinct mounted filesystems with their usage.
fn read_partitions() -> Vec<PartitionInfo> {
    let Some(content) = sysfs::read("/proc/mounts") else {
        return Vec::new();
    };

    let mut seen = HashSet::new();
    let mut partitions: Vec<PartitionInfo> = content
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let device = fields.next()?;
            let mount = fields.next()?;
            let filesystem = fields.next()?;

            // Only block devices, and only the first mount of each path.
            if !device.starts_with("/dev/")
                || IGNORED_MOUNTS.iter().any(|i| mount.starts_with(i) || mount.contains(i))
                || !seen.insert(mount.to_string())
            {
                return None;
            }

            let (total_bytes, free_bytes) = disk_usage(mount)?;
            let used_bytes = total_bytes.saturating_sub(free_bytes);
            Some(PartitionInfo {
                mount: mount.to_string(),
                filesystem: filesystem.to_string(),
                total_bytes,
                free_bytes,
                used_bytes,
                percent_used: ((used_bytes as f32 / total_bytes as f32) * 100.0).clamp(0.0, 100.0),
            })
        })
        .collect();

    // Root first, the rest alphabetically.
    partitions.sort_by(|a, b| (a.mount != "/", &a.mount).cmp(&(b.mount != "/", &b.mount)));
    partitions
}

/// Total and available bytes for a mount point, via `statvfs(3)`.
///
/// Returns `None` for pseudo-filesystems that report a zero-sized volume.
fn disk_usage(mount: &str) -> Option<(u64, u64)> {
    let path = CString::new(mount).ok()?;
    let mut stat = MaybeUninit::<libc::statvfs>::uninit();

    // SAFETY: `path` is a valid NUL-terminated string and `stat` is only read
    // once the call reports success, which means the kernel initialised it.
    let stat = unsafe {
        if libc::statvfs(path.as_ptr(), stat.as_mut_ptr()) != 0 {
            return None;
        }
        stat.assume_init()
    };

    let block_size = if stat.f_frsize > 0 { stat.f_frsize } else { stat.f_bsize } as u64;
    let total = stat.f_blocks as u64 * block_size;
    // `f_bavail` excludes root-reserved blocks, so it matches what users can fill.
    let free = stat.f_bavail as u64 * block_size;

    (total > 0).then_some((total, free))
}
