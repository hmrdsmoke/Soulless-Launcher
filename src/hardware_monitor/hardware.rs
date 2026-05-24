// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

use sysinfo::{CpuExt, System, SystemExt};
use crate::hardware_monitor::HISTORY;

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct HardwareState {
    // CPU
    pub cpu_usage:       f32,
    pub cpu_freq_mhz:    u64,
    pub cpu_temp:        Option<f32>,
    pub cpu_history:     Vec<f32>,

    // GPU
    pub gpu_usage:       Option<u32>,
    pub gpu_temp:        Option<u32>,
    pub gpu_clock_mhz:   Option<u32>,
    pub gpu_vram_used:   Option<u64>,
    pub gpu_vram_total:  Option<u64>,
    pub gpu_history:     Vec<f32>,

    // RAM
    pub ram_used_mb:     u64,
    pub ram_total_mb:    u64,
    pub ram_pct:         f32,
    pub ram_history:     Vec<f32>,

    // sysinfo handle — not Clone, so we box it
    #[allow(clippy::box_collection)]
    sys: Box<System>,
}

impl HardwareState {
    pub fn new() -> Self {
        let mut sys = Box::new(System::new_all());
        sys.refresh_all();

        Self {
            cpu_usage:      0.0,
            cpu_freq_mhz:   0,
            cpu_temp:       None,
            cpu_history:    vec![0.0; HISTORY],

            gpu_usage:      None,
            gpu_temp:       None,
            gpu_clock_mhz:  None,
            gpu_vram_used:  None,
            gpu_vram_total: None,
            gpu_history:    vec![0.0; HISTORY],

            ram_used_mb:    0,
            ram_total_mb:   0,
            ram_pct:        0.0,
            ram_history:    vec![0.0; HISTORY],

            sys,
        }
    }

    /// Sample all hardware sources and push new data points.
    pub fn tick(&mut self) {
        self.sys.refresh_cpu();
        self.sys.refresh_memory();

        // ── CPU ──────────────────────────────────────────────────────────────
        let cpus = self.sys.cpus();
        let n = cpus.len();

        self.cpu_usage = if n > 0 {
            cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / n as f32
        } else {
            0.0
        };

        self.cpu_freq_mhz = if n > 0 {
            cpus.iter().map(|c| c.frequency()).sum::<u64>() / n as u64
        } else {
            0
        };

        self.cpu_temp = read_cpu_temp();
        push_capped(&mut self.cpu_history, self.cpu_usage);

        // ── RAM ──────────────────────────────────────────────────────────────
        self.ram_used_mb  = self.sys.used_memory()  / 1_048_576;
        self.ram_total_mb = self.sys.total_memory() / 1_048_576;
        self.ram_pct = if self.ram_total_mb > 0 {
            self.ram_used_mb as f32 / self.ram_total_mb as f32 * 100.0
        } else {
            0.0
        };
        push_capped(&mut self.ram_history, self.ram_pct);

        // ── GPU (NVML) ────────────────────────────────────────────────────────
        if let Some(gpu) = read_gpu_nvml() {
            self.gpu_usage      = Some(gpu.usage);
            self.gpu_temp       = Some(gpu.temp);
            self.gpu_clock_mhz  = Some(gpu.clock_mhz);
            self.gpu_vram_used  = Some(gpu.vram_used_mb);
            self.gpu_vram_total = Some(gpu.vram_total_mb);
            push_capped(&mut self.gpu_history, gpu.usage as f32);
        }
    }
}

impl Default for HardwareState {
    fn default() -> Self {
        Self::new()
    }
}

// ── GPU snapshot ──────────────────────────────────────────────────────────────

struct GpuSnapshot {
    usage:        u32,
    temp:         u32,
    clock_mhz:    u32,
    vram_used_mb: u64,
    vram_total_mb: u64,
}

fn read_gpu_nvml() -> Option<GpuSnapshot> {
    use nvml_wrapper::Nvml;
    use nvml_wrapper::enum_wrappers::device::{Clock, ClockId, TemperatureSensor};

    // Init NVML fresh each tick — lightweight on Linux, avoids stale handles.
    let nvml   = Nvml::init().ok()?;
    let device = nvml.device_by_index(0).ok()?;

    let usage     = device.utilization_rates().map(|u| u.gpu).unwrap_or(0);
    let temp      = device.temperature(TemperatureSensor::Gpu).unwrap_or(0);
    let clock_mhz = device.clock(Clock::Graphics, ClockId::Current).unwrap_or(0);

    let mem = device.memory_info().ok()?;
    let vram_used_mb  = mem.used  / 1_048_576;
    let vram_total_mb = mem.total / 1_048_576;

    Some(GpuSnapshot { usage, temp, clock_mhz, vram_used_mb, vram_total_mb })
}

// ── CPU temp via sysfs coretemp / k10temp ─────────────────────────────────────

fn read_cpu_temp() -> Option<f32> {
    let entries = std::fs::read_dir("/sys/class/hwmon").ok()?;

    for entry in entries.flatten() {
        let path      = entry.path();
        let name_path = path.join("name");
        let name      = std::fs::read_to_string(&name_path).ok()?;
        let name      = name.trim();

        // Intel = coretemp, AMD = k10temp
        if name == "coretemp" || name == "k10temp" {
            let temp_path = path.join("temp1_input");
            if let Ok(raw) = std::fs::read_to_string(&temp_path) {
                if let Ok(millideg) = raw.trim().parse::<f32>() {
                    return Some(millideg / 1000.0);
                }
            }
        }
    }

    None
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn push_capped(v: &mut Vec<f32>, value: f32) {
    v.push(value);
    if v.len() > HISTORY {
        v.remove(0);
    }
}

// === DONE ===
// HardwareState: CPU usage/freq/temp, GPU usage/temp/clock/vram, RAM used/total/pct :: done
// tick(): refreshes sysinfo, reads sysfs CPU temp, reads NVML GPU stats :: done
// push_capped(): rolls HISTORY-length histories for CPU, GPU, RAM :: done
// read_cpu_temp(): walks /sys/class/hwmon, handles coretemp + k10temp :: done
// read_gpu_nvml(): NVML init per tick, device 0, usage/temp/clock/vram :: done