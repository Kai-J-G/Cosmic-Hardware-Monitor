use std::fs;
use std::process::Command;

use super::types::GpuInfo;

pub fn read_gpu_info() -> Option<GpuInfo> {
    // 1. Try AMD GPU via sysfs & hwmon
    if let Some(amd_gpu) = read_amd_gpu() {
        return Some(amd_gpu);
    }

    // 2. Try NVIDIA GPU via nvidia-smi
    if let Some(nvidia_gpu) = read_nvidia_gpu() {
        return Some(nvidia_gpu);
    }

    // 3. Try Intel GPU via hwmon
    if let Some(intel_gpu) = read_intel_gpu() {
        return Some(intel_gpu);
    }

    None
}

fn read_amd_gpu() -> Option<GpuInfo> {
    let mut edge_temp = None;
    let mut junction_temp = None;
    let mut mem_temp = None;
    let mut power_draw_watts = None;
    let mut power_cap_watts = None;
    let mut fan_rpm = None;
    let mut clock_mhz = None;
    let mut found_hwmon = false;

    // Search hwmon for amdgpu
    if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
        for entry in entries.flatten() {
            let path = entry.path();
            let name_path = path.join("name");
            if let Ok(name) = fs::read_to_string(&name_path) {
                if name.trim() == "amdgpu" {
                    found_hwmon = true;

                    // Temperatures
                    if let Ok(t) = read_sysfs_f32(&path.join("temp1_input")) {
                        edge_temp = Some(t / 1000.0);
                    }
                    if let Ok(t) = read_sysfs_f32(&path.join("temp2_input")) {
                        junction_temp = Some(t / 1000.0);
                    }
                    if let Ok(t) = read_sysfs_f32(&path.join("temp3_input")) {
                        mem_temp = Some(t / 1000.0);
                    }

                    // Power
                    if let Ok(p) = read_sysfs_f32(&path.join("power1_average")) {
                        power_draw_watts = Some(p / 1_000_000.0);
                    } else if let Ok(p) = read_sysfs_f32(&path.join("power1_input")) {
                        power_draw_watts = Some(p / 1_000_000.0);
                    }

                    if let Ok(p) = read_sysfs_f32(&path.join("power1_cap")) {
                        power_cap_watts = Some(p / 1_000_000.0);
                    }

                    // Fan
                    if let Ok(f) = read_sysfs_u32(&path.join("fan1_input")) {
                        fan_rpm = Some(f);
                    }

                    // Clocks
                    if let Ok(c) = read_sysfs_u64(&path.join("freq1_input")) {
                        clock_mhz = Some((c / 1_000_000) as u32);
                    }
                    break;
                }
            }
        }
    }

    if !found_hwmon {
        return None;
    }

    // Search drm for GPU utilization and VRAM
    let mut utilization_percent = 0;
    let mut vram_used_bytes = 0;
    let mut vram_total_bytes = 0;
    let mut gpu_name = "AMD Radeon GPU".to_string();

    if let Ok(drm_entries) = fs::read_dir("/sys/class/drm") {
        for entry in drm_entries.flatten() {
            let path = entry.path();
            let dev_path = path.join("device");
            let busy_path = dev_path.join("gpu_busy_percent");
            if busy_path.exists() {
                if let Ok(u) = read_sysfs_u32(&busy_path) {
                    utilization_percent = u.min(100);
                }
                if let Ok(used) = read_sysfs_u64(&dev_path.join("mem_info_vram_used")) {
                    vram_used_bytes = used;
                }
                if let Ok(total) = read_sysfs_u64(&dev_path.join("mem_info_vram_total")) {
                    vram_total_bytes = total;
                }

                if clock_mhz.is_none() {
                    if let Ok(content) = fs::read_to_string(dev_path.join("pp_dpm_sclk")) {
                        for line in content.lines() {
                            if line.ends_with('*') {
                                let parts: Vec<&str> = line.split_whitespace().collect();
                                if parts.len() >= 2 {
                                    let num_str = parts[1].trim_end_matches("Mhz").trim_end_matches("MHz");
                                    if let Ok(mhz) = num_str.parse::<u32>() {
                                        clock_mhz = Some(mhz);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }

                if let Ok(pname) = fs::read_to_string(dev_path.join("product_name")) {
                    let trimmed = pname.trim();
                    if !trimmed.is_empty() {
                        gpu_name = trimmed.to_string();
                    }
                }
                break;
            }
        }
    }

    if gpu_name == "AMD Radeon GPU" {
        if let Some(name) = detect_gpu_name_from_lspci("AMD") {
            gpu_name = name;
        }
    }

    Some(GpuInfo {
        name: gpu_name,
        edge_temp: edge_temp.unwrap_or(35.0),
        junction_temp,
        mem_temp,
        utilization_percent,
        clock_mhz,
        vram_used_bytes,
        vram_total_bytes,
        power_draw_watts,
        power_cap_watts,
        fan_rpm,
    })
}

fn read_nvidia_gpu() -> Option<GpuInfo> {
    let output = Command::new("nvidia-smi")
        .args([
            "--query-gpu=temperature.gpu,name,utilization.gpu,memory.used,memory.total,power.draw,clocks.current.graphics",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next()?;
    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
    if parts.len() < 6 {
        return None;
    }

    let edge_temp: f32 = parts[0].parse().unwrap_or(35.0);
    let name = parts[1].to_string();
    let utilization_percent: u32 = parts[2].parse().unwrap_or(0);
    let vram_used_mb: u64 = parts[3].parse().unwrap_or(0);
    let vram_total_mb: u64 = parts[4].parse().unwrap_or(0);
    let power_draw_watts: Option<f32> = parts[5].parse().ok();
    let clock_mhz: Option<u32> = parts.get(6).and_then(|s| s.parse().ok());

    Some(GpuInfo {
        name,
        edge_temp,
        junction_temp: None,
        mem_temp: None,
        utilization_percent,
        clock_mhz,
        vram_used_bytes: vram_used_mb * 1024 * 1024,
        vram_total_bytes: vram_total_mb * 1024 * 1024,
        power_draw_watts,
        power_cap_watts: None,
        fan_rpm: None,
    })
}

fn read_intel_gpu() -> Option<GpuInfo> {
    let mut edge_temp = None;
    let mut found = false;

    if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(name) = fs::read_to_string(path.join("name")) {
                if name.trim().contains("i915") || name.trim().contains("xe") {
                    found = true;
                    if let Ok(t) = read_sysfs_f32(&path.join("temp1_input")) {
                        edge_temp = Some(t / 1000.0);
                    }
                    break;
                }
            }
        }
    }

    if !found && edge_temp.is_none() {
        return None;
    }

    let mut gpu_name = "Intel Integrated Graphics".to_string();
    if let Some(name) = detect_gpu_name_from_lspci("Intel") {
        gpu_name = name;
    }

    Some(GpuInfo {
        name: gpu_name,
        edge_temp: edge_temp.unwrap_or(35.0),
        junction_temp: None,
        mem_temp: None,
        utilization_percent: 0,
        clock_mhz: None,
        vram_used_bytes: 0,
        vram_total_bytes: 0,
        power_draw_watts: None,
        power_cap_watts: None,
        fan_rpm: None,
    })
}

fn detect_gpu_name_from_lspci(vendor: &str) -> Option<String> {
    let output = Command::new("lspci").output().ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if (line.contains("VGA compatible controller") || line.contains("3D controller"))
            && line.contains(vendor)
        {
            if let Some(idx) = line.find(": ") {
                let desc = &line[idx + 2..];
                return Some(desc.trim().to_string());
            }
        }
    }
    None
}

fn read_sysfs_f32(path: &std::path::Path) -> Result<f32, ()> {
    let content = fs::read_to_string(path).map_err(|_| ())?;
    content.trim().parse::<f32>().map_err(|_| ())
}

fn read_sysfs_u32(path: &std::path::Path) -> Result<u32, ()> {
    let content = fs::read_to_string(path).map_err(|_| ())?;
    content.trim().parse::<u32>().map_err(|_| ())
}

fn read_sysfs_u64(path: &std::path::Path) -> Result<u64, ()> {
    let content = fs::read_to_string(path).map_err(|_| ())?;
    content.trim().parse::<u64>().map_err(|_| ())
}
