// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// launcher/src/hardware_monitor/hardware.rs
// Hardware sampling - CPU/GPU temps, frequencies, and history.

use sysinfo::System;
use crate::hardware_monitor::HISTORY;
use std::sync::OnceLock;

/// Process-wide NVML handle. init() dlopens the driver and handshakes —
/// far too heavy to repeat every 2s tick. Attempted once, outcome cached
/// (including failure: a box with no NVIDIA driver stays that way for
/// the session).
static NVML: OnceLock<Option<nvml_wrapper::Nvml>> = OnceLock::new();

fn nvml() -> Option<&'static nvml_wrapper::Nvml> {
    NVML.get_or_init(|| nvml_wrapper::Nvml::init().ok()).as_ref()
}

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
        // Cheap by design: System isn't Clone, and new_all()+refresh_all()
        // enumerates every process on the box — a landmine if any iced code
        // ever clones monitor state. A bare System::new() suffices: the
        // next tick() refreshes what it reads.
        let sys = System::new();
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

        // NVML first (NVIDIA proprietary), then sysfs (amdgpu/nouveau/i915).
        // sysfs fills what the driver exposes and leaves the rest None —
        // amdgpu gives temp+clocks+VRAM, nouveau usually temp only.
        if let Some(gpu) = read_gpu_nvml().or_else(read_gpu_sysfs) {
            self.gpu_temp_c     = gpu.temp;
            self.gpu_clock_mhz  = gpu.core_clock_mhz;
            self.gpu_mem_clock  = gpu.mem_clock_mhz;
            self.gpu_vram_used  = gpu.vram_used_mb;
            self.gpu_vram_total = gpu.vram_total_mb;
            if let Some(t) = gpu.temp {
                push_capped(&mut self.gpu_history, t as f32);
            }
        }
    }
}

impl Default for HardwareState {
    fn default() -> Self { Self::new() }
}

/// Every field optional: NVML supplies all of them, sysfs supplies whatever
/// the driver exposes. A None field renders as "—" in the view.
struct GpuSnapshot {
    temp:           Option<u32>,
    core_clock_mhz: Option<u32>,
    mem_clock_mhz:  Option<u32>,
    vram_used_mb:   Option<u64>,
    vram_total_mb:  Option<u64>,
}

fn read_gpu_nvml() -> Option<GpuSnapshot> {
    use nvml_wrapper::enum_wrappers::device::{Clock, ClockId, TemperatureSensor};

    let device = nvml()?.device_by_index(0).ok()?;

    // ok() not unwrap_or(0): a failed read is unknown, not zero. Zero is a
    // legitimate clock value on an idle card, so conflating them would show
    // a real "0 MHz" indistinguishable from a broken sensor.
    let temp           = device.temperature(TemperatureSensor::Gpu).ok();
    let core_clock_mhz = device.clock(Clock::Graphics, ClockId::Current).ok();
    let mem_clock_mhz  = device.clock(Clock::Memory,   ClockId::Current).ok();
    let mem            = device.memory_info().ok();

    Some(GpuSnapshot {
        temp,
        core_clock_mhz,
        mem_clock_mhz,
        vram_used_mb:  mem.as_ref().map(|m| m.used  / 1_048_576),
        vram_total_mb: mem.as_ref().map(|m| m.total / 1_048_576),
    })
}

// ── sysfs GPU fallback — amdgpu / nouveau / i915 ──────────────────────────────
//
// Drivers that aren't NVIDIA-proprietary expose stats through the kernel's
// DRM and hwmon interfaces instead of a userspace library. Paths:
//
//   /sys/class/drm/card*/device/hwmon/hwmon*/name          driver name
//   /sys/class/drm/card*/device/hwmon/hwmon*/temp1_input   temp, millidegrees
//   /sys/class/drm/card*/device/hwmon/hwmon*/freq1_input   core clock, Hz
//   /sys/class/drm/card*/device/hwmon/hwmon*/freq2_input   mem clock, Hz
//   /sys/class/drm/card*/device/mem_info_vram_used         VRAM used, bytes
//   /sys/class/drm/card*/device/mem_info_vram_total        VRAM total, bytes
//
// amdgpu exposes all of it; nouveau typically temp only; i915 varies.

/// Drivers we recognise, most-preferred first. A box with integrated + discrete
/// graphics enumerates both, and /sys/class/drm order is not guaranteed — match
/// on driver name rather than taking whichever card readdir hands over first,
/// so the discrete card wins instead of the iGPU.
const GPU_DRIVER_PRIORITY: &[&str] = &["amdgpu", "nouveau", "radeon", "i915", "xe"];

fn read_gpu_sysfs() -> Option<GpuSnapshot> {
    let mut best: Option<(usize, std::path::PathBuf, std::path::PathBuf)> = None;

    // continue, not `?` — same lesson as read_cpu_temp(): one unreadable node
    // must not abort the scan, and enumeration order varies by boot.
    let cards = std::fs::read_dir("/sys/class/drm").ok()?;
    for card in cards.flatten() {
        let device = card.path().join("device");
        let Ok(hwmons) = std::fs::read_dir(device.join("hwmon")) else {
            continue;
        };

        for hwmon in hwmons.flatten() {
            let hwmon_path = hwmon.path();
            let Ok(name) = std::fs::read_to_string(hwmon_path.join("name")) else {
                continue;
            };
            let name = name.trim();

            let Some(rank) = GPU_DRIVER_PRIORITY.iter().position(|d| *d == name) else {
                continue;
            };

            // Lower rank wins; first match at a given rank is kept.
            if best.as_ref().is_none_or(|(r, _, _)| rank < *r) {
                best = Some((rank, hwmon_path, device.clone()));
            }
        }
    }

    let (_, hwmon, device) = best?;

    let temp = read_u64(&hwmon.join("temp1_input"))
        .map(|milli| (milli / 1000) as u32);

    // hwmon freq*_input is Hz; the UI wants MHz.
    let core_clock_mhz = read_u64(&hwmon.join("freq1_input"))
        .map(|hz| (hz / 1_000_000) as u32);
    let mem_clock_mhz = read_u64(&hwmon.join("freq2_input"))
        .map(|hz| (hz / 1_000_000) as u32);

    let vram_used_mb  = read_u64(&device.join("mem_info_vram_used"))
        .map(|b| b / 1_048_576);
    let vram_total_mb = read_u64(&device.join("mem_info_vram_total"))
        .map(|b| b / 1_048_576);

    // A card that yielded nothing readable is the same as no card at all.
    if temp.is_none()
        && core_clock_mhz.is_none()
        && mem_clock_mhz.is_none()
        && vram_used_mb.is_none()
        && vram_total_mb.is_none()
    {
        return None;
    }

    Some(GpuSnapshot {
        temp,
        core_clock_mhz,
        mem_clock_mhz,
        vram_used_mb,
        vram_total_mb,
    })
}

fn read_u64(path: &std::path::Path) -> Option<u64> {
    std::fs::read_to_string(path).ok()?.trim().parse::<u64>().ok()
}

fn read_cpu_temp() -> Option<f32> {
    let entries = std::fs::read_dir("/sys/class/hwmon").ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        // continue, not `?` — one unreadable hwmon node aborted the whole
        // scan, and enumeration order varies by boot: CPU temp vanished
        // intermittently depending on which node came up first.
        let Ok(name) = std::fs::read_to_string(path.join("name")) else {
            continue;
        };
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
// GpuSnapshot fields all Option — partial data from sysfs renders as "—" :: done
// read_gpu_nvml(): .ok() not unwrap_or(0) — failed read is unknown, not zero :: done
// read_gpu_sysfs(): amdgpu/nouveau/radeon/i915/xe via DRM + hwmon :: done
// GPU_DRIVER_PRIORITY: driver-name match, discrete wins over iGPU :: done
// sysfs scan uses continue-not-? so one bad node can't abort it :: done
// hwmon freq*_input is Hz → MHz; temp1_input is millidegrees → °C :: done
// All-None snapshot returns None — no card is the same as no data :: done

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
// read_gpu_nvml(): persistent NVML handle (OnceLock), device 0 :: done
// read_cpu_temp(): sysfs coretemp + k10temp :: done

// === DONE ===
// HardwareState: CPU usage/freq/temp, GPU usage/temp/clock/vram, RAM used/total/pct :: done
// tick(): refreshes sysinfo, reads sysfs CPU temp, reads NVML GPU stats :: done
// push_capped(): rolls HISTORY-length histories for CPU, GPU, RAM :: done
// read_cpu_temp(): walks /sys/class/hwmon, handles coretemp + k10temp :: done
// read_gpu_nvml(): persistent NVML handle, device 0, usage/temp/clock/vram :: done