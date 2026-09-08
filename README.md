# TempTyle for COSMIC Desktop

A native hardware and thermal monitor applet for the **COSMIC Desktop Environment**, written in Rust with [System76's `libcosmic`](https://github.com/pop-os/libcosmic).

Inspired by [TempTyle](https://github.com/neojakey/TempTyle).

---

## Features

- **Panel applet**: lives in your COSMIC panel or dock, showing a thermometer tinted by thermal state and the current CPU temperature.
- **Direct kernel telemetry**: reads straight from `sysfs` and `procfs` — no daemon, no polling helper, no dependency on `lm_sensors`. All of `/sys/class/hwmon` is scanned once per refresh and shared between the CPU, GPU and drive collectors.
- **Overview**: CPU, GPU, memory and disk dials, each a shortcut into its own tab, over an expandable panel with network throughput, the CPU user/sys/idle split, load averages, VRAM, drive temperatures and uptime.
- **CPU tab**: package temperature, utilisation and average clock; power draw from RAPL where available; process, thread and file-descriptor counts; a temperature curve; and a per-core grid with thermal bars and live clocks.
  - AMD (`k10temp` / `zenpower`): Tctl/Tdie package and per-CCD temperatures.
  - Intel (`coretemp`): package and per-core temperatures.
- **GPU tab**: utilisation, clock, power draw, edge and junction temperatures, plus VRAM usage.
  - AMD via `sysfs` and the DRM device, NVIDIA via `nvidia-smi`, Intel via `i915`/`xe` hwmon.
- **Memory tab**: in use and available, physical and swap breakdown, committed and cached, with a utilisation history chart.
- **Storage tab**: whole-disk read and write throughput, and every mounted partition with its free space.
- **Settings**: follow the desktop theme or pin light/dark, switch between °C and °F, and pick a refresh interval from 1s to 5s.

---

## Build & Installation

### Prerequisites (Arch / CachyOS)

```bash
sudo pacman -S --needed rust just
```

### Install

```bash
just install
```

This builds the optimised release binary and installs:

- Executable to `~/.local/bin/cosmic-ext-temptyle`
- Desktop file to `~/.local/share/applications/`
- Metainfo to `~/.local/share/metainfo/`
- Icons to `~/.local/share/icons/hicolor/scalable/apps/`

Run `just uninstall` to remove all of it, and `just check` to compile and test without installing.

### Adding to the COSMIC panel

1. Open **COSMIC Settings → Desktop → Panel → Applets**.
2. Click **Add Applet** and select **Hardware Monitor**.
3. Click the panel icon to open the popup.

---

## Layout

```
src/
  main.rs            entry point
  app.rs             applet state, messages and the update loop
  config.rs          persisted settings (cosmic-config)
  hardware/          reads the machine
    sysfs.rs         sysfs/procfs helpers, hwmon enumeration, rate metering
    cpu.rs           model, temperatures, utilisation, clocks, power
    gpu.rs           AMD / NVIDIA / Intel telemetry
    storage.rs       drive temperatures, throughput, partitions
    system.rs        uptime, load, network, memory
    types.rs         the data the views render
  views/             renders the popup
    panel.rs         panel button and popup surface
    style.rs         shared container styling
    fmt.rs           shared value formatting
    overview.rs cores.rs gpu.rs memory.rs storage.rs settings.rs
    sparkline.rs circular_gauge.rs vertical_bar.rs
```

---

## License

MIT
