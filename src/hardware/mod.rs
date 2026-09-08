//! Reads the machine's hardware state from Linux `sysfs` and `procfs`.
//!
//! [`HardwareCollector`] owns everything that must persist between refreshes:
//! the cumulative counters that only mean something as a delta, and the
//! hardware identities that are resolved once rather than re-probed each tick.

pub mod cpu;
pub mod gpu;
pub mod storage;
pub mod sysfs;
pub mod system;
pub mod types;

use cpu::CpuCollector;
use gpu::GpuCollector;
use storage::StorageCollector;
use system::SystemCollector;
use types::HardwareSnapshot;

pub struct HardwareCollector {
    cpu: CpuCollector,
    gpu: GpuCollector,
    storage: StorageCollector,
    system: SystemCollector,
}

impl HardwareCollector {
    pub fn new() -> Self {
        Self {
            cpu: CpuCollector::new(),
            gpu: GpuCollector::new(&sysfs::hwmon_chips()),
            storage: StorageCollector::default(),
            system: SystemCollector::default(),
        }
    }

    /// Takes one reading of the whole machine.
    pub fn collect(&mut self) -> HardwareSnapshot {
        // CPU, GPU and drive sensors all live under hwmon, so scan it once and
        // share the result rather than walking the directory three times.
        let chips = sysfs::hwmon_chips();
        let (storage, storage_metrics) = self.storage.collect(&chips);
        let (system, memory) = self.system.collect();

        HardwareSnapshot {
            cpu: self.cpu.collect(&chips),
            gpu: self.gpu.collect(&chips),
            storage,
            storage_metrics,
            memory,
            system,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The collectors must survive whatever sensors this machine happens to
    /// expose, and utilisation only appears once two samples exist.
    #[test]
    fn collects_two_consecutive_snapshots() {
        let mut collector = HardwareCollector::new();
        let _ = collector.collect();
        std::thread::sleep(std::time::Duration::from_millis(200));
        let snapshot = collector.collect();

        assert!(snapshot.cpu.package_temp > 0.0, "no CPU temperature");
        assert!(!snapshot.cpu.cores.is_empty(), "no CPU cores");
        assert!(snapshot.memory.total_bytes > 0, "no memory total");
        assert!((0.0..=100.0).contains(&snapshot.cpu.overall_usage));
    }
}
