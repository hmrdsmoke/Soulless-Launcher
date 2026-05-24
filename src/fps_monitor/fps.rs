// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

use std::path::PathBuf;
use std::fs;
use crate::fps_monitor::HISTORY;

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FpsState {
    /// Most recent FPS reading (0 = no game active)
    pub fps:           f32,
    /// Most recent frametime in ms
    pub frametime_ms:  f32,
    /// Rolling 1% low over the last HISTORY samples
    pub fps_1_low:     f32,
    /// Rolling average over the last HISTORY samples
    pub fps_avg:       f32,
    /// Frametime history for the sparkline
    pub ft_history:    Vec<f32>,

    // Internal tracking
    last_log:    Option<PathBuf>,
    last_size:   u64,
    all_samples: Vec<(f32, f32)>, // (fps, frametime_ms)
}

impl FpsState {
    pub fn new() -> Self {
        Self {
            fps:          0.0,
            frametime_ms: 0.0,
            fps_1_low:    0.0,
            fps_avg:      0.0,
            ft_history:   vec![0.0; HISTORY],
            last_log:     None,
            last_size:    0,
            all_samples:  Vec::new(),
        }
    }

    /// Re-read the latest MangoHud CSV and refresh stats.
    pub fn tick(&mut self) {
        let Some(log_path) = latest_mangohud_log() else {
            return;
        };

        // Switched to a new game — reset
        if self.last_log.as_ref() != Some(&log_path) {
            self.all_samples.clear();
            self.last_size = 0;
            self.last_log  = Some(log_path.clone());
        }

        // Only re-parse if the file has grown
        let current_size = fs::metadata(&log_path)
            .map(|m| m.len())
            .unwrap_or(0);

        if current_size <= self.last_size {
            return;
        }
        self.last_size = current_size;

        if let Some(rows) = parse_mangohud_csv(&log_path) {
            self.all_samples = rows;
        }

        // ── Compute stats from the last HISTORY samples ───────────────────
        let window: Vec<(f32, f32)> = self.all_samples
            .iter()
            .rev()
            .take(HISTORY)
            .cloned()
            .collect();

        if window.is_empty() {
            return;
        }

        let (latest_fps, latest_ft) = window[0];
        self.fps          = latest_fps;
        self.frametime_ms = latest_ft;

        let fps_vals: Vec<f32> = window.iter().map(|(f, _)| *f).collect();
        self.fps_avg = fps_vals.iter().sum::<f32>() / fps_vals.len() as f32;

        // 1% low — average of the bottom 1% of FPS samples (worst frames)
        let mut sorted = fps_vals.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n_low = ((sorted.len() as f32 * 0.01).ceil() as usize).max(1);
        self.fps_1_low = sorted[..n_low].iter().sum::<f32>() / n_low as f32;

        // Frametime sparkline — push latest, keep HISTORY entries
        push_capped(&mut self.ft_history, latest_ft);
    }
}

impl Default for FpsState {
    fn default() -> Self {
        Self::new()
    }
}

// ── MangoHud log helpers ──────────────────────────────────────────────────────

fn mangohud_log_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".local/share/MangoHud")
}

/// Returns the most recently modified .csv in the MangoHud log directory.
fn latest_mangohud_log() -> Option<PathBuf> {
    let entries = fs::read_dir(mangohud_log_dir()).ok()?;

    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("csv") {
            continue;
        }
        if let Ok(meta) = entry.metadata() {
            if let Ok(modified) = meta.modified() {
                if best.as_ref().map_or(true, |(t, _)| modified > *t) {
                    best = Some((modified, path));
                }
            }
        }
    }

    best.map(|(_, p)| p)
}

/// Parses a MangoHud CSV and returns (fps, frametime_ms) rows.
///
/// MangoHud CSV format:
///   Lines starting with '#' are comments.
///   First non-comment line is the header: fps,frametime,...
///   Subsequent lines are data rows.
fn parse_mangohud_csv(path: &PathBuf) -> Option<Vec<(f32, f32)>> {
    use std::io::{BufRead, BufReader};

    let file   = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);

    let mut fps_col: Option<usize> = None;
    let mut ft_col:  Option<usize> = None;
    let mut rows = Vec::new();

    for line in reader.lines().flatten() {
        if line.starts_with('#') {
            continue;
        }

        let fields: Vec<&str> = line.split(',').collect();

        // Header row — locate column indices once
        if fps_col.is_none() {
            fps_col = fields.iter().position(|f| f.trim() == "fps");
            ft_col  = fields.iter().position(|f| f.trim() == "frametime");
            continue;
        }

        if let (Some(fi), Some(ti)) = (fps_col, ft_col) {
            let fps = fields.get(fi).and_then(|v| v.trim().parse::<f32>().ok());
            let ft  = fields.get(ti).and_then(|v| v.trim().parse::<f32>().ok());
            if let (Some(f), Some(t)) = (fps, ft) {
                rows.push((f, t));
            }
        }
    }

    if rows.is_empty() { None } else { Some(rows) }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn push_capped(v: &mut Vec<f32>, value: f32) {
    v.push(value);
    if v.len() > HISTORY {
        v.remove(0);
    }
}

// === DONE ===
// FpsState: fps, frametime_ms, fps_1_low, fps_avg, ft_history :: done
// tick(): watches latest MangoHud CSV, re-parses only on file growth :: done
// Game switch detection: resets samples when log path changes :: done
// 1% low: bottom 1% of HISTORY FPS samples, min 1 :: done
// ft_history: HISTORY-length frametime sparkline :: done
// parse_mangohud_csv(): skips '#' comments, finds fps/frametime columns :: done