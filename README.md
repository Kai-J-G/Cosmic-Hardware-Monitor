<img src="data/icons/temptyle.png" alt="TempTyle" width="128">

# COSMIC HARDWARE MONITOR

**A native hardware and thermal monitor applet for the COSMIC desktop.**

Lives in your panel, shows the temperature at a glance, and opens into a full
monitor for CPU, GPU, memory and storage.

Written in Rust with [libcosmic](https://github.com/pop-os/libcosmic).
Reads straight from the kernel — no daemon, no helper service, no `lm_sensors` dependency.

</div>

---

## Contents

- [What it shows](#what-it-shows)
- [Requirements](#requirements)
- [Install](#install)
- [Add it to your panel](#add-it-to-your-panel)
- [Updating](#updating)
- [Uninstalling](#uninstalling)
- [Building manually](#building-manually)
- [Troubleshooting](#troubleshooting)
- [Project layout](#project-layout)
- [License](#license)

---

## What it shows

**In the panel** — a thermometer icon tinted by thermal state (cyan → emerald →
gold → amber → red) next to the current CPU temperature.

**In the popup** — four dials for CPU, GPU, memory and disk, each a shortcut
into its own tab, over an expandable panel with network throughput, the CPU
user/sys/idle split, load averages, VRAM, drive temperatures and uptime.

| Tab | Readings |
| --- | --- |
| **CPU** | Package temperature, utilisation, average clock, power draw, process/thread/handle counts, a temperature curve, and a per-core grid with thermal bars and live clocks |
| **GPU** | Utilisation, clock, power draw, edge and junction temperatures, VRAM usage, temperature curve |
| **Memory** | In use and available, physical and swap breakdown, committed and cached, utilisation history |
| **Storage** | Whole-disk read/write throughput, and every mounted partition with its free space |
| **Settings** | Follow the desktop theme or pin light/dark, °C or °F, refresh interval from 1s to 5s |

**Hardware support**

| | Source |
| --- | --- |
| AMD CPU | `k10temp` / `zenpower` — Tctl/Tdie package and per-CCD temperatures |
| Intel CPU | `coretemp` — package and per-core temperatures |
| AMD GPU | `sysfs` + the DRM device |
| NVIDIA GPU | `nvidia-smi` |
| Intel GPU | `i915` / `xe` hwmon |
| Storage | `nvme` / `drivetemp` hwmon, `/proc/diskstats`, `statvfs` |

---

## Requirements

**To run**

- A **COSMIC desktop** session (this is a panel applet — it has no standalone window).
- Linux, with the usual `/sys` and `/proc` filesystems.
- `libxkbcommon` — already present in any COSMIC session.

**To build**

| Package | Why |
| --- | --- |
| A recent stable **Rust** toolchain | Built and tested with 1.98; edition 2021 |
| **git** | `libcosmic` is fetched as a git dependency |
| **just** | Runs the install recipes |
| **libxkbcommon** headers | Linked by the windowing layer |
| **pkgconf** / `pkg-config` | Used by dependency build scripts |

**Optional, improves what's shown**

- `pciutils` — supplies a readable GPU name when the driver doesn't expose one.
- `nvidia-smi` (ships with the NVIDIA driver) — required for NVIDIA telemetry.

---

## Install

### 1. Install the build prerequisites

<details open>
<summary><strong>Arch / CachyOS</strong></summary>

```bash
sudo pacman -S --needed rustup just git libxkbcommon pkgconf pciutils
```

If you've never used `rustup` before, pick a toolchain:

```bash
rustup default stable
```

</details>

<details>
<summary><strong>Fedora</strong></summary>

```bash
sudo dnf install cargo just git libxkbcommon-devel pkgconf-pkg-config pciutils
```

</details>

<details>
<summary><strong>Debian / Ubuntu</strong></summary>

```bash
sudo apt install cargo git libxkbcommon-dev pkg-config pciutils
```

`just` isn't packaged on older releases. If `apt install just` fails:

```bash
cargo install just
```

</details>

### 2. Clone and install

```bash
git clone https://github.com/Kai-J-G/COSMIC---Hardware-Monitor-applet.git
```

```bash
cd COSMIC---Hardware-Monitor-applet && just install
```

That's it. `just install` compiles an optimised release build and installs it
into your home directory — **no `sudo` required**.

> **The first build takes a while.** Cargo fetches `libcosmic` and its
> dependency tree from git and compiles the lot — expect several minutes, and
> a `target/` directory that grows to several GB (around 9 GB once both debug
> and release builds exist). Later builds take seconds, and `cargo clean`
> reclaims all of it.

### What gets installed

Everything lands under `~/.local`, so nothing touches system directories:

| File | Path |
| --- | --- |
| Executable | `~/.local/bin/cosmic-ext-temptyle` |
| Desktop entry | `~/.local/share/applications/com.github.neojakey.cosmic-ext-temptyle.desktop` |
| AppStream metainfo | `~/.local/share/metainfo/com.github.neojakey.cosmic-ext-temptyle.metainfo.xml` |
| Icons | `~/.local/share/icons/hicolor/scalable/apps/com.github.neojakey.cosmic-ext-temptyle{,-symbolic}.svg` |

The desktop entry's `Exec=` line is rewritten to the absolute path of the
installed binary, so the applet works regardless of your `PATH`.

<details>
<summary><strong>Installing somewhere else (packagers)</strong></summary>

The destination is controlled by `PREFIX`, which defaults to `~/.local`:

```bash
PREFIX=/usr/local just install
```

Note that the `install` recipe builds first. For a system-wide install, build
as your normal user and only elevate the copy step — running the whole recipe
under `sudo` would compile as root and leave a root-owned `target/` directory
that your user can no longer write to.

</details>

---

## Add it to your panel

1. Open **COSMIC Settings → Desktop → Panel → Applets**.
2. Click **Add Applet**.
3. Select **Hardware Monitor** and place it wherever you like — next to the
   clock works well.
4. Click the panel icon to open the popup.

If it doesn't appear in the applet list, see
[Troubleshooting](#troubleshooting) below.

---

## Updating

```bash
git pull && just install
```

The applet reads its settings from `cosmic-config`, so your theme, unit and
refresh interval survive reinstalls.

The panel keeps running the old binary until it restarts, so log out and back
in afterwards. To skip the logout, restart just the panel — `cosmic-session`
supervises it and brings it straight back:

```bash
pkill cosmic-panel
```

---

## Uninstalling

```bash
just uninstall
```

This removes every file listed above. Remove the applet from your panel first,
in **COSMIC Settings → Desktop → Panel → Applets**.

To also reclaim the build directory:

```bash
cargo clean
```

Your saved settings live in
`~/.config/cosmic/com.github.neojakey.cosmic-ext-temptyle/` and are left alone.
Delete that directory if you want them gone too:

```bash
rm -r ~/.config/cosmic/com.github.neojakey.cosmic-ext-temptyle
```

---

## Building manually

Every recipe in the `justfile`:

| Command | What it does |
| --- | --- |
| `just` | Same as `just build-release` |
| `just build-release` | Optimised build → `target/release/cosmic-ext-temptyle` |
| `just build-debug` | Fast, unoptimised build |
| `just run` | Runs the applet directly with `RUST_BACKTRACE=1` |
| `just check` | `cargo check` and `cargo test` |
| `just install` | Build, then install to `PREFIX` (default `~/.local`) |
| `just uninstall` | Remove every installed file |

Plain cargo works too:

```bash
cargo build --release
```

```bash
cargo test
```

The release profile enables thin LTO, a single codegen unit and symbol
stripping, so `just build-release` is noticeably slower than a debug build but
produces a much smaller binary.

---

## Troubleshooting

<details>
<summary><strong>The applet doesn't appear in the "Add Applet" list</strong></summary>

COSMIC scans for applets when the panel starts, so it won't see a
freshly-installed one until then. Logging out and back in fixes this in almost
every case. To skip the logout, restart just the panel — `cosmic-session`
supervises it and brings it straight back:

```bash
pkill cosmic-panel
```

If it's still missing, confirm the desktop entry landed in the right place:

```bash
ls ~/.local/share/applications/com.github.neojakey.cosmic-ext-temptyle.desktop
```

A missing file means `just install` didn't finish — re-run it and check for
build errors.

</details>

<details>
<summary><strong>CPU temperature reads exactly 40°C</strong></summary>

That's the fallback used when no CPU thermal sensor can be found — the applet
reads `k10temp`, `zenpower` and `coretemp` under `/sys/class/hwmon`. Check what
your kernel exposes:

```bash
for d in /sys/class/hwmon/hwmon*; do echo "$(basename $d): $(cat $d/name)"; done
```

If none of those drivers is listed, load the one for your CPU (`k10temp` for
AMD, `coretemp` for Intel) with `sudo modprobe`.

</details>

<details>
<summary><strong>Power draw says "Estimating…"</strong></summary>

Package power comes from the kernel's RAPL energy counter. Some distributions
restrict it to root as a side-channel mitigation, in which case the applet
can't read it. Check:

```bash
ls -l /sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj
```

If the mode is `-r--------`, the counter is root-only on your system and the
field will stay unavailable. It never affects any other reading.

</details>

<details>
<summary><strong>The GPU tab says no GPU was detected</strong></summary>

- **AMD** — needs the `amdgpu` driver loaded and an `amdgpu` entry in
  `/sys/class/hwmon`.
- **NVIDIA** — needs `nvidia-smi` on your `PATH`; it ships with the proprietary
  driver. Nouveau exposes no telemetry.
- **Intel** — needs `i915` or `xe` hwmon; integrated graphics report a
  temperature only.

The vendor is probed once at start-up, so restart the applet after installing a
driver.

</details>

<details>
<summary><strong>My GPU has an unhelpful name</strong></summary>

The name comes from the DRM device, falling back to `lspci`. Install
`pciutils` if `lspci` is missing.

</details>

<details>
<summary><strong>No drive temperature for a SATA disk</strong></summary>

NVMe drives are handled by the `nvme` driver out of the box. SATA drives need
the `drivetemp` module:

```bash
sudo modprobe drivetemp
```

Add `drivetemp` to `/etc/modules-load.d/` to make it stick across reboots.

</details>

---

## Project layout

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
data/                desktop entry, AppStream metainfo, icons
```

All of `/sys/class/hwmon` is scanned once per refresh and shared between the
CPU, GPU and storage collectors; hardware identity and other values that can't
change while the applet runs are resolved once at start-up.

---

## Credits

Inspired by [TempTyle](https://github.com/neojakey/TempTyle) by neojakey.

## License

[MIT](LICENSE)
