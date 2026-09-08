//! CPU utilisation, from the cumulative counters in `/proc/stat`.
//!
//! The kernel does not report a percentage; it reports how many jiffies (ticks
//! of the scheduler clock) every CPU has spent in each state since boot. A
//! percentage is therefore a comparison of two readings: how much each counter
//! grew between them, as a share of the total growth.
//!
//! That is why [`UsageTracker`] has to remember the previous reading, and why
//! the very first refresh after start-up reports no load at all.

use std::collections::HashMap;

use crate::hardware::sysfs;

/// Remembers the previous `/proc/stat` reading so the next one can be compared
/// against it.
#[derive(Default)]
pub struct UsageTracker {
    previous_overall: Option<CpuTimes>,
    previous_cores: HashMap<usize, CpuTimes>,
}

impl UsageTracker {
    /// Takes a reading and returns the utilisation since the previous one.
    ///
    /// `/proc/stat` opens with an aggregate line and then one line per CPU:
    ///
    /// ```text
    /// cpu   9432 118 3021 884301 512 88 61 0
    /// cpu0   612   9  201  55210  33  7  4 0
    /// cpu1   588  11  195  55402  29  6  3 0
    /// intr  ...          ← everything after the cpu lines is something else
    /// ```
    pub fn read(&mut self) -> Usage {
        let Some(contents) = sysfs::read("/proc/stat") else {
            return Usage::default();
        };

        let mut usage = Usage::default();

        for line in contents.lines() {
            let mut fields = line.split_whitespace();

            let Some(name) = fields.next() else {
                continue;
            };
            // The cpu lines come first, so anything else means we are done.
            let Some(which_cpu) = name.strip_prefix("cpu") else {
                break;
            };

            let Some(times) = CpuTimes::parse(fields) else {
                continue;
            };

            if which_cpu.is_empty() {
                // The bare "cpu" line covers the whole machine, and is the only
                // one we break down into user, system and idle.
                if let Some(previous) = self.previous_overall.replace(times) {
                    usage = times.usage_since(previous);
                }
            } else if let Ok(index) = which_cpu.parse::<usize>() {
                // "cpu0", "cpu1", ... — for these we keep only the total.
                if let Some(previous) = self.previous_cores.insert(index, times) {
                    usage.per_core.insert(index, times.usage_since(previous).overall);
                }
            }
        }

        usage
    }
}

/// Jiffies spent in each state, as reported by one `/proc/stat` cpu line.
#[derive(Default, Clone, Copy)]
pub struct CpuTimes {
    user: u64,
    system: u64,
    idle: u64,
}

impl CpuTimes {
    /// Parses the counters that follow a `cpu` line's name.
    ///
    /// The kernel writes them in a fixed order, and we group the eight we care
    /// about into the three categories the UI shows:
    ///
    /// ```text
    /// cpu  12345 678 9012 345678 901 23 45 6
    ///      user  nice system idle iowait irq softirq steal
    /// ```
    ///
    /// Returns `None` for a line too short to be a real cpu line.
    fn parse<'a>(fields: impl Iterator<Item = &'a str>) -> Option<Self> {
        let counters: Vec<u64> = fields.take(8).map(|f| f.parse().unwrap_or(0)).collect();

        // The first four are always present; the rest were added over time and
        // may be missing on an old kernel, so read them defensively.
        if counters.len() < 4 {
            return None;
        }
        let counter = |index: usize| counters.get(index).copied().unwrap_or(0);

        let (user, nice) = (counter(0), counter(1));
        let (system, idle, iowait) = (counter(2), counter(3), counter(4));
        let (irq, softirq, steal) = (counter(5), counter(6), counter(7));

        Some(Self {
            // Nice time is still time spent running user code.
            user: user + nice,
            // Interrupt and stolen time are all work the CPU did for someone.
            system: system + irq + softirq + steal,
            // Waiting on I/O is time the CPU had nothing else to do.
            idle: idle + iowait,
        })
    }

    fn total(self) -> u64 {
        self.user + self.system + self.idle
    }

    /// Compares this reading against an earlier one to get percentages.
    ///
    /// The counters are cumulative since boot, so what matters is how much
    /// each grew over the interval, as a share of the total growth.
    fn usage_since(self, previous: Self) -> Usage {
        let elapsed = self.total().saturating_sub(previous.total());

        // No jiffies passed — too soon since the last reading to say anything.
        if elapsed == 0 {
            return Usage::default();
        }

        let share_of_interval = |now: u64, before: u64| {
            (now.saturating_sub(before) as f32 / elapsed as f32) * 100.0
        };
        let idle = share_of_interval(self.idle, previous.idle);

        Usage {
            // Whatever wasn't idle was work.
            overall: (100.0 - idle).clamp(0.0, 100.0),
            user: share_of_interval(self.user, previous.user),
            system: share_of_interval(self.system, previous.system),
            idle,
            per_core: HashMap::new(),
        }
    }
}

/// Utilisation percentages for one refresh, all 0–100.
pub struct Usage {
    pub overall: f32,
    pub user: f32,
    pub system: f32,
    pub idle: f32,
    /// Per-CPU utilisation, keyed by index. Empty on the first refresh.
    pub per_core: HashMap<usize, f32>,
}

impl Default for Usage {
    fn default() -> Self {
        Self { overall: 0.0, user: 0.0, system: 0.0, idle: 100.0, per_core: HashMap::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cpu_stat_fields() {
        // user nice system idle iowait irq softirq steal
        let times = CpuTimes::parse("100 10 50 800 20 5 5 0".split_whitespace()).unwrap();

        assert_eq!(times.user, 110);
        assert_eq!(times.system, 60);
        assert_eq!(times.idle, 820);
        assert_eq!(times.total(), 990);
    }

    #[test]
    fn rejects_truncated_cpu_stat_lines() {
        assert!(CpuTimes::parse("1 2 3".split_whitespace()).is_none());
    }

    #[test]
    fn usage_is_the_non_idle_share_of_the_interval() {
        let previous = CpuTimes { user: 0, system: 0, idle: 0 };
        let now = CpuTimes { user: 20, system: 5, idle: 75 };
        let usage = now.usage_since(previous);

        assert_eq!(usage.overall, 25.0);
        assert_eq!(usage.user, 20.0);
        assert_eq!(usage.system, 5.0);
        assert_eq!(usage.idle, 75.0);
    }

    #[test]
    fn identical_samples_report_no_usage() {
        let times = CpuTimes { user: 10, system: 10, idle: 10 };
        assert_eq!(times.usage_since(times).idle, 100.0);
    }
}
