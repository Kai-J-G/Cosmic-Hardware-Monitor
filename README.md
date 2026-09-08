# TempTyle for COSMIC Desktop

A native System Hardware and Thermal Monitor applet for **CachyOS** and the **COSMIC Desktop Environment**, written in Rust using [System76's `libcosmic`](https://github.com/pop-os/libcosmic).

Inspired by [TempTyle](https://github.com/neojakey/TempTyle).

---

## Features

- **Panel Applet Integration**: Lives directly in your COSMIC panel or dock with a live temperature icon and readout.
- **Microsecond Direct Linux Telemetry**: Reads directly from Linux sysfs (`/sys/class/hwmon`, `/sys/class/drm`, `/proc/stat`, `/proc/meminfo`), with 0% idle CPU overhead and tiny memory footprint (~24MB RSS).
- **AMD, Intel & NVIDIA Support**:
  - **AMD CPU**: CCD & Package/Tctl temperatures (`k10temp`/`zenpower`), per-core frequency, and per-core utilization.
  - **AMD GPU**: Edge, Junction/Hotspot, and Memory temperatures, GPU utilization %, VRAM used/total, power draw, and fan speed.
  - **NVIDIA GPU**: Automatic fallback to `nvidia-smi` telemetry.
  - **Intel CPU / GPU**: `coretemp` and `i915`/`xe` sensors.
  - **Storage**: NVMe composite temperatures, sub-sensor temps, and critical threshold alerts.
  - **Cooling Fans**: Real-time RPM tachometer monitoring across motherboard and GPU headers with active/stopped state detection.
  - **System & RAM**: Memory usage breakdown, CachyOS kernel/specs, uptime, load averages, and DDR5 RAM SPD thermal sensors (`spd5118`).
- **Interactive UI**:
  - **Dynamic Thermal State Badge**: Color-coded based on temperature (Cool Cyan `<45°C`, Optimal Green `45-64°C`, Warm Amber `65-79°C`, Critical Red `80°C+`).
  - **Live Sparkline Canvas**: Real-time 60fps temperature trend line chart with grid lines and gradient fill.
  - **Vertical Core Bars**: Hardware core thermal gauges inspired by TempTyle.
  - **One-Click Unit Toggle**: Seamlessly switch between `°C` and `°F`.

---

## Build & Installation

### Prerequisites (Arch / CachyOS)

Ensure Rust and `just` are available:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sudo pacman -S --needed just
```

### Install Applet to User Environment

Run:
```bash
just install
```

This compiles the optimized release binary and installs:
- Executable to `~/.local/bin/cosmic-ext-temptyle`
- Desktop file to `~/.local/share/applications/com.github.neojakey.cosmic-ext-temptyle.desktop`
- Metainfo to `~/.local/share/metainfo/com.github.neojakey.cosmic-ext-temptyle.metainfo.xml`
- Icons to `~/.local/share/icons/hicolor/scalable/apps/`

### Adding to COSMIC Panel

1. Open **COSMIC Settings** -> **Desktop** -> **Panel** -> **Applets**.
2. Click **Add Applet**.
3. Select **TempTyle** and position it wherever you prefer (e.g. next to the Time or Weather applets).
4. Click on the panel icon to open the full hardware monitoring popup!

---

## License

MIT
