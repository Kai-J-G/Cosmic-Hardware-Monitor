# Cosmic Hardware Monitor — English source strings.
#
# To translate: copy this file to i18n/<language>/ and translate the values on
# the right of each `=`. Leave the ids on the left alone, and keep the
# { $placeholders } intact — they are filled in at runtime.

app-title = Hardware Monitor
back-to-overview = ← Overview

## Overview

cpu = CPU
gpu = GPU
memory = Memory
disk = Disk

network = Network
upload = Upload
download = Download

cpu-usage = CPU Usage
usage-user = user
usage-system = sys
usage-idle = idle

load-average = Load avg
load-1m = 1m
load-5m = 5m
load-15m = 15m

gpu-usage = GPU Usage
gpu-memory-used = Memory Used
gpu-memory-size = Memory Size
gpu-absent = Integrated graphics or GPU idle

storage = Storage
uptime = Uptime ›

## CPU tab

temperature = Temperature
load-utilization = Load / Utilization
clock-speed = Clock Speed
cpu-temperature-curve = CPU Temperature Curve

power-draw = Power Draw
power-draw-label = Power Draw:
power-estimating = Estimating…
processes-label = Processes:
threads-label = Threads:
handles-label = Handles:

# { $count } is the number of logical CPUs.
all-cores = All { $count } Cores

## GPU tab

graphics = Graphics
gpu-unavailable = No dedicated GPU detected or GPU metrics unavailable.
utilization = Utilization
not-available = N/A
gpu-temperature-curve = GPU Temperature Curve
# { $temperature } is an already-formatted reading such as "35°".
gpu-junction = Junction: { $temperature }
# { $percent } is the share of video memory in use.
vram-usage = VRAM Usage: { $percent }%
# { $used } and { $total } are formatted sizes such as "15.92 GB".
vram-used-of-total = { $used } / { $total }

## Memory tab

memory-title = Memory & Swap Metrics
memory-in-use = In Use
memory-available = Available
# { $percent } is the share of physical memory in use.
memory-percent-of-ram = { $percent }% of RAM
memory-percent-free = { $percent }% free
memory-history = RAM Utilization History
physical-memory = Physical Memory: { $percent }%
# { $used } and { $total } are formatted sizes such as "7.02 GB".
memory-in-use-of-total = { $used } in use / { $total } total
memory-committed = Committed
memory-cached = Cached
swap-used = Swap Used
swap-available = Swap Available
swap-percent-used = { $percent }% used
# { $total } is a formatted size.
swap-of-total = of { $total }

## Storage tab

storage-title = Storage & Partition Metrics
read-speed = Read Speed
write-speed = Write Speed
partitions-title = Mounted Partitions & Free Space
partitions-none = No mounted partitions detected.
partitions-sandboxed =
    Partition usage is unavailable in the Flatpak build, which cannot see the
    host's filesystems. Throughput above is still accurate.
# { $percent } is how full the partition is; { $free } and { $total } are sizes.
partition-summary = { $percent }%  ·  { $free } free of { $total }

## Settings

settings = Settings

theme-title = Theme Adaptation
theme-description =
    Choose whether Hardware Monitor follows your desktop theme or stays locked
    to a light or dark appearance.
theme-system = System / Adapt to Desktop Theme
theme-system-description = Automatically switches between dark and light to match COSMIC
theme-dark = Black (Always Dark)
theme-dark-description = Deep dark background with high-contrast emerald and cyan accents
theme-light = White (Always Light)
theme-light-description = Clean light background with crisp dark typography

unit-title = Temperature Unit
unit-celsius = Celsius (°C)
unit-fahrenheit = Fahrenheit (°F)

interval-title = Refresh Interval
interval-1s = 1s (Fast)
interval-2s = 2s (Normal)
interval-3s = 3s
interval-5s = 5s (Battery-friendly)

## Units and formatting

unit-megabytes = { $value } MB
unit-gigabytes = { $value } GB
unit-kb-per-second = { $value } KB/s
unit-mb-per-second = { $value } MB/s
unit-watts = { $value } W
unit-megahertz = { $value } MHz
unit-gigahertz = { $value } GHz
unit-percent = { $value }%

# Uptime, at the coarsest unit that still says something useful.
uptime-minutes = { $minutes } minutes
uptime-hours = { $hours } hours, { $minutes } minutes
uptime-days = { $days }d, { $hours }h, { $minutes }m
