// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/system_monitor/stats.rs
// System stats sampling - CPU/RAM/GPU/disk history state.

use crate::system_monitor::HISTORY;

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct StatsState {
    pub cpu_history:  Vec<f32>,
    pub ram_history:  Vec<f32>,
    pub gpu_history:  Vec<f32>,
    pub disk_history: Vec<f32>,

    pub cpu_pct:  f32,
    pub ram_pct:  f32,
    pub gpu_pct:  f32,
    pub disk_pct: f32,

    // previous /proc/stat values for delta CPU calculation
    prev_idle:  u64,
    prev_total: u64,
    // ticks until the next df refresh — see tick()'s Disk section
    disk_tick_countdown: u32,
}

impl Default for StatsState {
    fn default() -> Self {
        Self {
            cpu_history:  vec![0.0; HISTORY],
            ram_history:  vec![0.0; HISTORY],
            gpu_history:  vec![0.0; HISTORY],
            disk_history: vec![0.0; HISTORY],
            cpu_pct:      0.0,
            ram_pct:      0.0,
            gpu_pct:      0.0,
            disk_pct:     0.0,
            prev_idle:    0,
            prev_total:   0,
            disk_tick_countdown: 0,
        }
    }
}

impl StatsState {
    pub fn new() -> Self {
        let mut s = Self::default();
        // Seed prev values so first tick gives a real delta
        if let Some((idle, total)) = read_cpu_raw() {
            s.prev_idle  = idle;
            s.prev_total = total;
        }
        s
    }

    pub fn tick(&mut self) {
        // ── CPU ──────────────────────────────────────────────────────────
        if let Some((idle, total)) = read_cpu_raw() {
            let d_total = total.saturating_sub(self.prev_total);
            let d_idle  = idle.saturating_sub(self.prev_idle);

            if d_total > 0 {
                self.cpu_pct = (1.0 - d_idle as f32 / d_total as f32) * 100.0;
                self.cpu_pct = self.cpu_pct.clamp(0.0, 100.0);
            }

            self.prev_idle  = idle;
            self.prev_total = total;
        }
        push_capped(&mut self.cpu_history, self.cpu_pct);

        // ── RAM ──────────────────────────────────────────────────────────
        self.ram_pct = read_ram_pct().unwrap_or(self.ram_pct);
        push_capped(&mut self.ram_history, self.ram_pct);

        // ── GPU ──────────────────────────────────────────────────────────
        self.gpu_pct = read_gpu_pct().unwrap_or(self.gpu_pct);
        push_capped(&mut self.gpu_history, self.gpu_pct);

        // ── Disk ─────────────────────────────────────────────────────────
        // df is a fork+exec to learn a number that moves by the hour —
        // refresh every 30th tick (~30s visible), reuse the cached value
        // between. History still gets a point per tick so the graph scrolls.
        if self.disk_tick_countdown == 0 {
            self.disk_pct = read_disk_pct().unwrap_or(self.disk_pct);
            self.disk_tick_countdown = 30;
        } else {
            self.disk_tick_countdown -= 1;
        }
        push_capped(&mut self.disk_history, self.disk_pct);
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn push_capped(v: &mut Vec<f32>, value: f32) {
    v.push(value);
    if v.len() > HISTORY {
        v.remove(0);
    }
}

/// Returns (idle_jiffies, total_jiffies) from /proc/stat cpu line.
fn read_cpu_raw() -> Option<(u64, u64)> {
    let data = std::fs::read_to_string("/proc/stat").ok()?;
    let line = data.lines().find(|l| l.starts_with("cpu "))?;

    let fields: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .map(|f| f.parse().unwrap_or(0))
        .collect();

    if fields.len() < 4 {
        return None;
    }

    // idle = fields[3], iowait = fields[4] (also idle-ish)
    let idle  = fields[3] + fields.get(4).copied().unwrap_or(0);
    let total = fields.iter().sum();

    Some((idle, total))
}

/// Returns RAM usage as a percentage from /proc/meminfo.
fn read_ram_pct() -> Option<f32> {
    let data = std::fs::read_to_string("/proc/meminfo").ok()?;
    let mut total: u64 = 0;
    let mut available: u64 = 0;

    for line in data.lines() {
        if line.starts_with("MemTotal:") {
            total = parse_kb(line)?;
        } else if line.starts_with("MemAvailable:") {
            available = parse_kb(line)?;
        }
    }

    if total == 0 {
        return None;
    }

    let used = total.saturating_sub(available);
    Some((used as f32 / total as f32 * 100.0).clamp(0.0, 100.0))
}

fn parse_kb(line: &str) -> Option<u64> {
    line.split_whitespace().nth(1)?.parse().ok()
}

/// Returns GPU usage % by trying nvidia-smi, then AMD sysfs.
/// Returns None if no GPU info is available.
fn read_gpu_pct() -> Option<f32> {
    // ── NVIDIA ────────────────────────────────────────────────────────────
    if let Ok(out) = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=utilization.gpu", "--format=csv,noheader,nounits"])
        .output()
        && out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            if let Ok(v) = s.trim().parse::<f32>() {
                return Some(v.clamp(0.0, 100.0));
            }
        }

    // ── AMD (amdgpu sysfs) ────────────────────────────────────────────────
    // /sys/class/drm/card*/device/gpu_busy_percent
    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let path = entry.path().join("device/gpu_busy_percent");
            if let Ok(data) = std::fs::read_to_string(&path)
                && let Ok(v) = data.trim().parse::<f32>() {
                    return Some(v.clamp(0.0, 100.0));
                }
        }
    }

    // ── Intel (i915 sysfs) ────────────────────────────────────────────────
    // /sys/class/drm/card*/gt/gt0/rc6_enable (not a utilisation metric)
    // Better: /sys/class/drm/card*/gt_cur_freq_mhz ratio — not reliable.
    // Fall back to 0 rather than showing nothing.
    Some(0.0)
}

/// Returns disk usage % for the filesystem containing "/".
fn read_disk_pct() -> Option<f32> {
    let out = std::process::Command::new("df")
        .args(["--output=pcent", "/"])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&out.stdout);

    // Output is:
    //   Use%
    //   42%
    for line in stdout.lines().skip(1) {
        let pct_str = line.trim().trim_end_matches('%');
        if let Ok(v) = pct_str.parse::<f32>() {
            return Some(v.clamp(0.0, 100.0));
        }
    }

    None
}

// === DONE ===
// StatsState: rolling HISTORY-sample histories for CPU/RAM/GPU/Disk :: done
// read_cpu_raw(): /proc/stat delta-based CPU% :: done
// read_ram_pct(): /proc/meminfo MemTotal - MemAvailable :: done
// read_gpu_pct(): tries nvidia-smi, then AMD sysfs, falls back to 0 :: done
// read_disk_pct(): df --output=pcent / :: done
// tick(): updates all four stats and pushes to histories :: done