use crate::error::Result;
use std::fs;

#[derive(Debug, Clone)]
pub struct KernelVitals {
    pub load_avg_1m: f64,
    pub load_avg_5m: f64,
    pub load_avg_15m: f64,
    pub interrupts_per_sec: u64,
    pub context_switches_per_sec: u64,
    pub processes_running: u32,
    pub processes_blocked: u32,
}

impl KernelVitals {
    pub fn collect() -> Result<Self> {
        let loadavg = Self::read_loadavg()?;
        let stat = Self::read_stat()?;

        Ok(Self {
            load_avg_1m: loadavg.0,
            load_avg_5m: loadavg.1,
            load_avg_15m: loadavg.2,
            interrupts_per_sec: stat.0,
            context_switches_per_sec: stat.1,
            processes_running: loadavg.3,
            processes_blocked: loadavg.4,
        })
    }

    fn read_loadavg() -> Result<(f64, f64, f64, u32, u32)> {
        let content = fs::read_to_string("/proc/loadavg")?;
        let parts: Vec<&str> = content.split_whitespace().collect();

        if parts.len() < 5 {
            return Ok((0.0, 0.0, 0.0, 0, 0));
        }

        let load1 = parts[0].parse::<f64>().unwrap_or(0.0);
        let load5 = parts[1].parse::<f64>().unwrap_or(0.0);
        let load15 = parts[2].parse::<f64>().unwrap_or(0.0);

        // Parse "running/total" field
        let proc_parts: Vec<&str> = parts[3].split('/').collect();
        let running = if proc_parts.len() >= 1 {
            proc_parts[0].parse::<u32>().unwrap_or(0)
        } else {
            0
        };

        Ok((load1, load5, load15, running, 0))
    }

    fn read_stat() -> Result<(u64, u64)> {
        let content = fs::read_to_string("/proc/stat")?;

        let mut interrupts = 0u64;
        let mut context_switches = 0u64;

        for line in content.lines() {
            if line.starts_with("intr ") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    interrupts = value.parse::<u64>().unwrap_or(0);
                }
            } else if line.starts_with("ctxt ") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    context_switches = value.parse::<u64>().unwrap_or(0);
                }
            }
        }

        Ok((interrupts, context_switches))
    }
}
