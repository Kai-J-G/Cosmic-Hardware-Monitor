use std::fs;

use super::types::FanInfo;

pub fn read_fans_info() -> Vec<FanInfo> {
    let mut fans = Vec::new();

    let Ok(entries) = fs::read_dir("/sys/class/hwmon") else {
        return fans;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let chip_name = fs::read_to_string(path.join("name"))
            .unwrap_or_else(|_| "Unknown".to_string())
            .trim()
            .to_string();

        let Ok(files) = fs::read_dir(&path) else {
            continue;
        };

        for file in files.flatten() {
            let fname = file.file_name();
            let fname_str = fname.to_string_lossy();
            if fname_str.starts_with("fan") && fname_str.ends_with("_input") {
                let prefix = &fname_str[..fname_str.len() - 6]; // e.g. "fan1"
                let label = fs::read_to_string(path.join(format!("{prefix}_label")))
                    .unwrap_or_else(|_| {
                        if chip_name.contains("gpu") {
                            "GPU Fan".to_string()
                        } else {
                            format!("{chip_name} {}", prefix.to_uppercase())
                        }
                    })
                    .trim()
                    .to_string();

                if let Ok(val_str) = fs::read_to_string(file.path()) {
                    if let Ok(rpm) = val_str.trim().parse::<u32>() {
                        fans.push(FanInfo {
                            chip: chip_name.clone(),
                            name: label,
                            rpm,
                            is_active: rpm > 0,
                        });
                    }
                }
            }
        }
    }

    fans.sort_by(|a, b| b.rpm.cmp(&a.rpm));
    fans
}
