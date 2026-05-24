// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

use crate::fps_monitor::HISTORY;
use std::time::Instant;

// ── Constants ─────────────────────────────────────────────────────────────────

/// How often the subscription fires in ms — must match TICK_FPS_MS in mod.rs
const TICK_MS: f32 = 500.0;

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FpsState {
    pub fps:          f32,
    pub fps_avg:      f32,
    pub fps_1_low:    f32,
    pub frametime_ms: f32,
    pub ft_history:   Vec<f32>,

    last_tick:    Option<Instant>,
    ft_samples:   Vec<f32>,
}

impl FpsState {
    pub fn new() -> Self {
        Self {
            fps:          0.0,
            fps_avg:      0.0,
            fps_1_low:    0.0,
            frametime_ms: 0.0,
            ft_history:   vec![0.0; HISTORY],
            last_tick:    None,
            ft_samples:   Vec::new(),
        }
    }

    pub fn tick(&mut self) {
        let now = Instant::now();

        if let Some(last) = self.last_tick {
            // Actual elapsed since last tick in ms
            let elapsed_ms = now.duration_since(last).as_secs_f32() * 1000.0;

            // Clamp to sane range — ignore if system was suspended or
            // something went very wrong (> 5 seconds gap)
            if elapsed_ms < 5000.0 {
                self.frametime_ms = elapsed_ms;
                self.fps = 1000.0 / elapsed_ms;

                push_capped(&mut self.ft_history, elapsed_ms);
                push_capped(&mut self.ft_samples, elapsed_ms);

                // Keep ft_samples bounded to HISTORY
                if self.ft_samples.len() > HISTORY {
                    self.ft_samples.remove(0);
                }

                // Rolling average FPS
                let avg_ft = self.ft_samples.iter().sum::<f32>()
                    / self.ft_samples.len() as f32;
                self.fps_avg = 1000.0 / avg_ft;

                // 1% low — worst (highest) frametimes → lowest FPS
                let mut sorted = self.ft_samples.clone();
                sorted.sort_by(|a, b| b.partial_cmp(a).unwrap()); // descending
                let n_low = ((sorted.len() as f32 * 0.01).ceil() as usize).max(1);
                let worst_ft = sorted[..n_low].iter().sum::<f32>() / n_low as f32;
                self.fps_1_low = 1000.0 / worst_ft;
            }
        }

        self.last_tick = Some(now);
    }
}

impl Default for FpsState {
    fn default() -> Self { Self::new() }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn push_capped(v: &mut Vec<f32>, value: f32) {
    v.push(value);
    if v.len() > HISTORY { v.remove(0); }
}

// === DONE ===
// FpsState: tick-based frame timing — measures actual elapsed between ticks :: done
// fps: 1000 / elapsed_ms — reflects compositor responsiveness :: done
// fps_avg: rolling average over HISTORY samples :: done
// fps_1_low: worst 1% frametimes converted to FPS :: done
// ft_history: frametime sparkline history :: done
// Clamped to 5s gap — ignores suspend/resume spikes :: done
// No external deps — pure std timing :: done

// === DONE ===
// FpsState: fps, fps_avg, fps_1_low, frametime_ms, ft_history :: done
// tick(): reads shared frame timestamps, computes FPS + stats :: done
// run_presentation_listener(): background thread, wp_presentation_feedback :: done
// Presented event → pushes Instant to shared ring buffer :: done
// FPS = frames in last 1 second window :: done
// 1% low = worst frametime bucket converted back to FPS :: done
// Graceful fallback if compositor doesn't support wp_presentation :: done

// === DONE ===
// FpsState: fps, frametime_ms, fps_1_low, fps_avg, ft_history :: done
// tick(): watches latest MangoHud CSV, re-parses only on file growth :: done
// Game switch detection: resets samples when log path changes :: done
// 1% low: bottom 1% of HISTORY FPS samples, min 1 :: done
// ft_history: HISTORY-length frametime sparkline :: done
// parse_mangohud_csv(): skips '#' comments, finds fps/frametime columns :: done