// GPL-3.0-or-later - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

use sysinfo::System;
use crate::hardware_monitor::HISTORY;

#[derive(Debug)]
pub struct HardwareState {
    pub cpu_temp_c:     Option<f32>,
    pub cpu_freq_mhz:   u64,
    pub cpu_history:    Vec<f32>,
    pub gpu_temp_c:     Option<u32>,
    pub gpu_clock_mhz:  Option<u32>,
    pub gpu_mem_clock:  Option<u32>,
    pub gpu_vram_used:  Option<u64>,
    pub gpu_vram_total: Option<u64>,
    pub gpu_history:    Vec<f32>,
    pub ram_freq_mhz:   Option<u32>,
    sys: System,
}

impl Clone for HardwareState {
    fn clone(&self) -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self {
            cpu_temp_c:     self.cpu_temp_c,
            cpu_freq_mhz:   self.cpu_freq_mhz,
            cpu_history:    self.cpu_history.clone(),
            gpu_temp_c:     self.gpu_temp_c,
            gpu_clock_mhz:  self.gpu_clock_mhz,
            gpu_mem_clock:  self.gpu_mem_clock,
            gpu_vram_used:  self.gpu_vram_used,
            gpu_vram_total: self.gpu_vram_total,
            gpu_history:    self.gpu_history.clone(),
            ram_freq_mhz:   self.ram_freq_mhz,
            sys,
        }
    }
}

impl HardwareState {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self {
            cpu_temp_c:     None,
            cpu_freq_mhz:   0,
            cpu_history:    vec![0.0; HISTORY],
            gpu_temp_c:     None,
            gpu_clock_mhz:  None,
            gpu_mem_clock:  None,
            gpu_vram_used:  None,
            gpu_vram_total: None,
            gpu_history:    vec![0.0; HISTORY],
            ram_freq_mhz:   read_ram_freq_cached(),
            sys,
        }
    }

    pub fn tick(&mut self) {
        self.sys.refresh_cpu_all();

        let cpus = self.sys.cpus();
        let n = cpus.len();
        self.cpu_freq_mhz = if n > 0 {
            cpus.iter().map(|c| c.frequency()).sum::<u64>() / n as u64
        } else { 0 };

        self.cpu_temp_c = read_cpu_temp();
        if let Some(t) = self.cpu_temp_c {
            push_capped(&mut self.cpu_history, t);
        }

        if let Some(gpu) = read_gpu_nvml() {
            self.gpu_temp_c    = Some(gpu.temp);
            self.gpu_clock_mhz = Some(gpu.core_clock_mhz);
            self.gpu_mem_clock  = Some(gpu.mem_clock_mhz);
            self.gpu_vram_used  = Some(gpu.vram_used_mb);
            self.gpu_vram_total = Some(gpu.vram_total_mb);
            push_capped(&mut self.gpu_history, gpu.temp as f32);
        }
    }
}

impl Default for HardwareState {
    fn default() -> Self { Self::new() }
}

struct GpuSnapshot {
    temp:           u32,
    core_clock_mhz: u32,
    mem_clock_mhz:  u32,
    vram_used_mb:   u64,
    vram_total_mb:  u64,
}

fn read_gpu_nvml() -> Option<GpuSnapshot> {
    use nvml_wrapper::Nvml;
    use nvml_wrapper::enum_wrappers::device::{Clock, ClockId, TemperatureSensor};

    let nvml   = Nvml::init().ok()?;
    let device = nvml.device_by_index(0).ok()?;

    let temp           = device.temperature(TemperatureSensor::Gpu).unwrap_or(0);
    let core_clock_mhz = device.clock(Clock::Graphics, ClockId::Current).unwrap_or(0);
    let mem_clock_mhz  = device.clock(Clock::Memory,   ClockId::Current).unwrap_or(0);
    let mem            = device.memory_info().ok()?;

    Some(GpuSnapshot {
        temp,
        core_clock_mhz,
        mem_clock_mhz,
        vram_used_mb:  mem.used  / 1_048_576,
        vram_total_mb: mem.total / 1_048_576,
    })
}

fn read_cpu_temp() -> Option<f32> {
    let entries = std::fs::read_dir("/sys/class/hwmon").ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = std::fs::read_to_string(path.join("name")).ok()?;
        let name = name.trim();
        if (name == "coretemp" || name == "k10temp")
            && let Ok(raw) = std::fs::read_to_string(path.join("temp1_input"))
                && let Ok(millideg) = raw.trim().parse::<f32>() {
                    return Some(millideg / 1000.0);
                }
    }
    None
}

// ── RAM frequency — cached dmidecode read ─────────────────────────────────────
//
// Reads RAM freq once via dmidecode, caches to ~/.cache/soulless/ram_freq.txt.
// pkexec only fires if the cache file doesn't exist yet — i.e. first launch only.

const RAM_FREQ_CACHE: &str = ".cache/soulless/ram_freq.txt";

pub fn read_ram_freq_cached() -> Option<u32> {
    // ── Try cache first ───────────────────────────────────────────────────────
    let cache_path = dirs::home_dir()?.join(RAM_FREQ_CACHE);

    if cache_path.exists()
        && let Ok(contents) = std::fs::read_to_string(&cache_path) {
            let trimmed = contents.trim();
            // "none" means we tried before and got nothing — don't prompt again
            if trimmed == "none" {
                return None;
            }
            if let Ok(mhz) = trimmed.parse::<u32>() {
                return Some(mhz);
            }
        }

    // ── Cache miss — run dmidecode once and save result ───────────────────────
    let mhz = run_dmidecode();

    // Ensure cache dir exists
    if let Some(parent) = cache_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // Write result or "none" so we never prompt again
    let contents = mhz
        .map(|m| m.to_string())
        .unwrap_or_else(|| "none".to_string());

    let _ = std::fs::write(&cache_path, contents);

    mhz
}

fn run_dmidecode() -> Option<u32> {
    // Try without root first — works on some systems
    let output = std::process::Command::new("dmidecode")
        .args(["-t", "17"])
        .output()
        .ok()?;

    if output.status.success() {
        return parse_dmidecode_speed(&output.stdout);
    }

    // Needs root — pkexec fires here (only on first launch ever)
    let output = std::process::Command::new("pkexec")
        .args(["dmidecode", "-t", "17"])
        .output()
        .ok()?;

    if output.status.success() {
        return parse_dmidecode_speed(&output.stdout);
    }

    eprintln!("RAM freq: dmidecode failed — will show — and not prompt again");
    None
}

fn parse_dmidecode_speed(stdout: &[u8]) -> Option<u32> {
    let text = String::from_utf8_lossy(stdout);
    let mut configured: Option<u32> = None;
    let mut speed:      Option<u32> = None;

    for line in text.lines() {
        let t = line.trim();
        if t.starts_with("Configured Memory Speed:")
            && let Some(mhz) = extract_mhz(t) {
                configured = Some(configured.unwrap_or(0).max(mhz));
            }
        if t.starts_with("Speed:") && !t.contains("Unknown")
            && let Some(mhz) = extract_mhz(t) {
                speed = Some(speed.unwrap_or(0).max(mhz));
            }
    }

    configured.or(speed)
}

fn extract_mhz(line: &str) -> Option<u32> {
    let after = line.split_once(':')?.1.trim();
    let num   = after.split_whitespace().next()?;
    num.parse::<u32>().ok().filter(|&n| n > 0 && n < 20_000)
}

fn push_capped(v: &mut Vec<f32>, value: f32) {
    v.push(value);
    if v.len() > HISTORY { v.remove(0); }
}

// === DONE ===
// read_ram_freq_cached(): reads ~/.cache/soulless/ram_freq.txt first :: done
// Cache miss → runs dmidecode once, writes result (or "none") to cache :: done
// "none" in cache = previous attempt failed — never prompts again :: done
// pkexec only fires on very first launch when cache doesn't exist :: done
// If user changes RAM speed (OC/BIOS): delete cache file to re-read :: done

// === DONE ===
// HardwareState: cpu temp+freq, gpu temp+clocks+vram, ram freq (dmidecode) :: done
// Manual Clone impl — System doesn't derive Clone :: done
// read_ram_freq_dmidecode(): plain then pkexec fallback, one-time at startup :: done
// parse_dmidecode_speed(): prefers Configured Memory Speed (XMP/OC) over Speed :: done

// === DONE ===
// MHz spelled out in full for GPU clock and RAM freq :: done
// CPU freq auto-scales: GHz if >= 1000 MHz, MHz otherwise :: done
// cpu col: label / temp°C (heat-coloured) / freq :: done
// gpu col: label / temp°C (heat-coloured) / core clock MHz :: done
// ram col: label / freq MHz (dmidecode, static) :: done

// === DONE ===
// Reworked: usage% removed — hardware monitor shows temp + freq only :: done
// CPU: freq_mhz (avg across cores) + temp_c (sysfs coretemp/k10temp) :: done
// GPU: temp_c + core_clock_mhz + mem_clock_mhz + vram used/total :: done
// RAM: freq_mhz read once from DMI type 17 entries (no-root sysfs path) :: done
// Sparkline histories now track temp, not usage :: done

// === DONE ===
// Fixed: CpuExt/SystemExt removed — sysinfo 0.39 uses methods directly on System :: done
// Fixed: refresh_cpu() → refresh_cpu_all() :: done
// Fixed: System doesn't impl Clone — manual Clone impl rebuilds System :: done
// Fixed: derive(Clone) removed from struct, manual impl added :: done
// read_gpu_nvml(): NVML init per tick, device 0 :: done
// read_cpu_temp(): sysfs coretemp + k10temp :: done

// === DONE ===
// HardwareState: CPU usage/freq/temp, GPU usage/temp/clock/vram, RAM used/total/pct :: done
// tick(): refreshes sysinfo, reads sysfs CPU temp, reads NVML GPU stats :: done
// push_capped(): rolls HISTORY-length histories for CPU, GPU, RAM :: done
// read_cpu_temp(): walks /sys/class/hwmon, handles coretemp + k10temp :: done
// read_gpu_nvml(): NVML init per tick, device 0, usage/temp/clock/vram :: done