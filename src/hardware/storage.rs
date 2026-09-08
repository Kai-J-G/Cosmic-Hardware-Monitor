use std::collections::HashSet;
use std::ffi::CString;
use std::fs;
use std::mem::MaybeUninit;
use std::time::Instant;

use super::types::{PartitionInfo, StorageInfo, StorageMetrics};

#[derive(Debug, Default)]
pub struct StorageCollector {
    last_sectors_read: u64,
    last_sectors_written: u64,
    last_time: Option<Instant>,
}

impl StorageCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn collect(&mut self) -> (Vec<StorageInfo>, StorageMetrics) {
        let drives = read_storage_info();
        let (read_kbs, write_kbs) = self.read_disk_io();
        let partitions = read_partitions();
        (drives, StorageMetrics { read_kbs, write_kbs, partitions })
    }

    fn read_disk_io(&mut self) -> (f32, f32) {
        let (sectors_read, sectors_written) = parse_diskstats_sectors();
        let now = Instant::now();
        let mut rates = (0.0f32, 0.0f32);

        if let Some(prev_time) = self.last_time {
            let dt = (now - prev_time).as_secs_f32();
            if dt > 0.05 {
                let r_diff = sectors_read.saturating_sub(self.last_sectors_read);
                let w_diff = sectors_written.saturating_sub(self.last_sectors_written);
                let r_bytes = r_diff * 512;
                let w_bytes = w_diff * 512;
                rates = ((r_bytes as f32 / dt) / 1024.0, (w_bytes as f32 / dt) / 1024.0);
            }
        }

        self.last_sectors_read = sectors_read;
        self.last_sectors_written = sectors_written;
        self.last_time = Some(now);
        rates
    }
}

fn parse_diskstats_sectors() -> (u64, u64) {
    let mut total_r = 0u64;
    let mut total_w = 0u64;

    if let Ok(content) = fs::read_to_string("/proc/diskstats") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 10 {
                continue;
            }
            let dev_name = parts[2];
            let is_root_disk = (dev_name.starts_with("nvme") && dev_name.contains('n') && !dev_name.contains('p'))
                || (dev_name.starts_with("sd") && dev_name.len() == 3);

            if is_root_disk {
                let r_sectors: u64 = parts[5].parse().unwrap_or(0);
                let w_sectors: u64 = parts[9].parse().unwrap_or(0);
                total_r += r_sectors;
                total_w += w_sectors;
            }
        }
    }
    (total_r, total_w)
}

pub fn read_partitions() -> Vec<PartitionInfo> {
    let mut list = Vec::new();
    let mut seen_mounts = HashSet::new();

    if let Ok(content) = fs::read_to_string("/proc/mounts") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                continue;
            }
            let dev = parts[0];
            let mount = parts[1];
            let fs_type = parts[2];

            if !dev.starts_with("/dev/") {
                continue;
            }
            if mount.starts_with("/var/")
                || mount == "/srv"
                || mount == "/root"
                || mount.contains("/var/lib/docker")
                || mount.contains("/var/lib/flatpak")
                || mount.contains("/snap")
            {
                continue;
            }
            if seen_mounts.contains(mount) {
                continue;
            }
            seen_mounts.insert(mount.to_string());

            let c_path = match CString::new(mount) {
                Ok(p) => p,
                Err(_) => continue,
            };

            let mut stat = MaybeUninit::<libc::statvfs>::uninit();
            let res = unsafe { libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) };
            if res == 0 {
                let stat = unsafe { stat.assume_init() };
                let block_size = if stat.f_frsize > 0 { stat.f_frsize } else { stat.f_bsize } as u64;
                let total_bytes = stat.f_blocks as u64 * block_size;
                let free_bytes = stat.f_bavail as u64 * block_size;
                let used_bytes = total_bytes.saturating_sub(free_bytes);
                if total_bytes > 0 {
                    let percent_used = (used_bytes as f32 / total_bytes as f32) * 100.0;
                    list.push(PartitionInfo {
                        mount: mount.to_string(),
                        filesystem: fs_type.to_string(),
                        total_bytes,
                        free_bytes,
                        used_bytes,
                        percent_used: percent_used.clamp(0.0, 100.0),
                    });
                }
            }
        }
    }

    list.sort_by(|a, b| {
        if a.mount == "/" {
            std::cmp::Ordering::Less
        } else if b.mount == "/" {
            std::cmp::Ordering::Greater
        } else {
            a.mount.cmp(&b.mount)
        }
    });

    list
}

pub fn read_storage_info() -> Vec<StorageInfo> {
    let mut storage_list = Vec::new();

    let Ok(entries) = fs::read_dir("/sys/class/hwmon") else {
        return storage_list;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let chip_name = fs::read_to_string(path.join("name"))
            .unwrap_or_else(|_| String::new())
            .trim()
            .to_string();

        if chip_name.starts_with("nvme") || chip_name.starts_with("drivetemp") {
            let mut composite_temp = None;
            let mut sensor1_temp = None;
            let mut sensor2_temp = None;
            let mut crit_temp = None;

            if let Ok(t) = read_sysfs_f32(&path.join("temp1_input")) {
                composite_temp = Some(t / 1000.0);
            }
            if let Ok(t) = read_sysfs_f32(&path.join("temp2_input")) {
                sensor1_temp = Some(t / 1000.0);
            }
            if let Ok(t) = read_sysfs_f32(&path.join("temp3_input")) {
                sensor2_temp = Some(t / 1000.0);
            }
            if let Ok(t) = read_sysfs_f32(&path.join("temp1_crit")) {
                crit_temp = Some(t / 1000.0);
            }

            if let Some(comp_temp) = composite_temp {
                let name = get_drive_model(&path, &chip_name);
                storage_list.push(StorageInfo {
                    name,
                    composite_temp: comp_temp,
                    sensor1_temp,
                    sensor2_temp,
                    crit_temp,
                });
            }
        }
    }

    storage_list
}

fn get_drive_model(hwmon_path: &std::path::Path, chip_name: &str) -> String {
    let dev_model = hwmon_path.join("device/model");
    if let Ok(m) = fs::read_to_string(&dev_model) {
        let trimmed = m.trim();
        if !trimmed.is_empty() {
            return format!("NVMe {}", trimmed);
        }
    }

    if chip_name.starts_with("nvme") {
        if let Ok(entries) = fs::read_dir("/sys/class/nvme") {
            for e in entries.flatten() {
                if let Ok(m) = fs::read_to_string(e.path().join("model")) {
                    let trimmed = m.trim();
                    if !trimmed.is_empty() {
                        return format!("NVMe {}", trimmed);
                    }
                }
            }
        }
    }

    format!("Storage ({chip_name})")
}

fn read_sysfs_f32(path: &std::path::Path) -> Result<f32, ()> {
    let content = fs::read_to_string(path).map_err(|_| ())?;
    content.trim().parse::<f32>().map_err(|_| ())
}
