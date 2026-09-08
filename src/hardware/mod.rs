pub mod cpu;
pub mod fans;
pub mod gpu;
pub mod storage;
pub mod system;
pub mod types;

use cpu::{read_cpu_info, CpuStatHistory};
use fans::read_fans_info;
use gpu::read_gpu_info;
use storage::StorageCollector;
use system::SystemCollector;
pub use types::*;

pub struct HardwareCollector {
    cpu_stat_history: CpuStatHistory,
    system_collector: SystemCollector,
    storage_collector: StorageCollector,
}

impl HardwareCollector {
    pub fn new() -> Self {
        Self {
            cpu_stat_history: CpuStatHistory::default(),
            system_collector: SystemCollector::new(),
            storage_collector: StorageCollector::new(),
        }
    }

    pub fn collect(&mut self) -> HardwareSnapshot {
        let cpu = read_cpu_info(&mut self.cpu_stat_history);
        let gpu = read_gpu_info();
        let fans = read_fans_info();
        let (storage, storage_metrics) = self.storage_collector.collect();
        let (system, memory) = self.system_collector.collect();

        // Collect all temperatures for summary calculations
        let mut all_temps = Vec::new();
        all_temps.push(cpu.package_temp);
        for c in &cpu.cores {
            all_temps.push(c.temp);
        }

        let gpu_temp = gpu.as_ref().map(|g| {
            all_temps.push(g.edge_temp);
            if let Some(j) = g.junction_temp {
                all_temps.push(j);
            }
            g.edge_temp
        });

        for s in &storage {
            all_temps.push(s.composite_temp);
        }

        for (_, t) in &system.spd_temps {
            all_temps.push(*t);
        }

        let max_temp = all_temps
            .iter()
            .copied()
            .fold(0.0f32, |acc, t| acc.max(t));
        let min_temp = all_temps
            .iter()
            .copied()
            .fold(999.0f32, |acc, t| acc.min(t));
        let avg_temp = if !all_temps.is_empty() {
            all_temps.iter().sum::<f32>() / all_temps.len() as f32
        } else {
            cpu.package_temp
        };

        // Primary temp: max of CPU package temp and GPU temp
        let primary_temp = match gpu_temp {
            Some(gt) => cpu.package_temp.max(gt),
            None => cpu.package_temp,
        };

        let thermal_status = ThermalStatus::from_celsius(primary_temp);
        let active_fans_count = fans.iter().filter(|f| f.is_active).count();

        HardwareSnapshot {
            summary: ThermalSummary {
                package_temp: cpu.package_temp,
                gpu_temp,
                max_temp,
                min_temp: if min_temp > 500.0 { 0.0 } else { min_temp },
                avg_temp,
                total_cores: cpu.cores.len(),
                active_fans_count,
                thermal_status,
            },
            cpu,
            gpu,
            fans,
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

    #[test]
    fn test_collect() {
        let mut collector = HardwareCollector::new();
        let _ = collector.collect();
        std::thread::sleep(std::time::Duration::from_millis(200));
        let snapshot = collector.collect();
        println!("CPU: {} ({:.1}°C, {}%)", snapshot.cpu.model, snapshot.cpu.package_temp, snapshot.cpu.overall_usage);
        println!("Cores detected: {}", snapshot.cpu.cores.len());
        for c in &snapshot.cpu.cores {
            println!("  {} -> {:.1}°C, {:.1}%", c.label, c.temp, c.usage_percent);
        }
        if let Some(gpu) = &snapshot.gpu {
            println!("GPU: {} (Edge: {:.1}°C, Junction: {:?}, VRAM: {}/{} bytes, Load: {}%)",
                gpu.name, gpu.edge_temp, gpu.junction_temp, gpu.vram_used_bytes, gpu.vram_total_bytes, gpu.utilization_percent);
        }
        for f in &snapshot.fans {
            println!("Fan: {} ({} RPM)", f.name, f.rpm);
        }
        for s in &snapshot.storage {
            println!("Drive: {} ({:.1}°C)", s.name, s.composite_temp);
        }
        println!("OS: {} | Host: {} | Kernel: {}", snapshot.system.os_name, snapshot.system.hostname, snapshot.system.kernel);
        println!("RAM: {:.1} GB / {:.1} GB ({:.1}%)",
            snapshot.system.mem_used_bytes as f64 / 1_073_741_824.0,
            snapshot.system.mem_total_bytes as f64 / 1_073_741_824.0,
            snapshot.system.mem_percent);
        for (name, t) in &snapshot.system.spd_temps {
            println!("SPD: {} = {:.1}°C", name, t);
        }
        println!("CPU Details: avg_freq: {} MHz, power: {:?} W, procs: {}, threads: {}, handles: {}",
            snapshot.cpu.avg_freq_mhz, snapshot.cpu.power_watts, snapshot.cpu.num_processes, snapshot.cpu.num_threads, snapshot.cpu.num_handles);
        if let Some(gpu) = &snapshot.gpu {
            println!("GPU Clock: {:?} MHz, Power: {:?} W", gpu.clock_mhz, gpu.power_draw_watts);
        }
        println!("Storage Metrics: read: {:.1} KB/s, write: {:.1} KB/s, partitions: {}",
            snapshot.storage_metrics.read_kbs, snapshot.storage_metrics.write_kbs, snapshot.storage_metrics.partitions.len());
        for p in &snapshot.storage_metrics.partitions {
            println!("  Mount: {} ({}) -> Total: {}, Used: {}, Free: {}, Pct: {:.1}%",
                p.mount, p.filesystem, p.total_bytes, p.used_bytes, p.free_bytes, p.percent_used);
        }
        println!("Memory Metrics: Used: {}, Avail: {}, Committed: {}, Cached: {}, SwapUsed: {}, SwapAvail: {}, Pct: {:.1}%",
            snapshot.memory.used_bytes, snapshot.memory.available_bytes, snapshot.memory.committed_bytes, snapshot.memory.cached_bytes,
            snapshot.memory.swap_used_bytes, snapshot.memory.swap_avail_bytes, snapshot.memory.percent);
        assert!(snapshot.cpu.package_temp > 0.0);
    }
}

