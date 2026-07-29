// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/fps_monitor/fps.rs
// FPS/frametime sampling and rolling history.

use crate::fps_monitor::HISTORY;
use std::time::Instant;

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

        let Some(last) = self.last_tick else {
            self.last_tick = Some(now);
            return;
        };

        let elapsed_ms = now.duration_since(last).as_secs_f32() * 1000.0;

        // Burst fold. wayland_frames() ORs two event sources (RedrawRequested
        // + the Wayland Frame callback) across every surface, with no surface
        // id on the payload — one painted frame can arrive as several ticks
        // with near-zero gaps, which read as thousands of fps. Anything under
        // the floor is the same frame echoing: don't advance last_tick, let
        // the gap keep accumulating from the frame that anchored it, and the
        // burst collapses into one sample. 2ms = 500fps ceiling; no display
        // this widget will meet presents faster.
        const MIN_FRAME_MS: f32 = 2.0;
        if elapsed_ms < MIN_FRAME_MS {
            return;
        }

        // Suspend clamp — the 5s+ gap of a sleep/resume isn't a frame. Reset
        // the anchor so the next real frame measures cleanly.
        if elapsed_ms >= 5000.0 {
            self.last_tick = Some(now);
            return;
        }

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
// Burst fold: sub-2ms gaps are the same frame echoing (dual event sources ×
// multiple surfaces, no id on payload) — folded into one sample :: done
// last_tick only advances on accepted frames or suspend reset :: done

// === DONE ===
// FpsState: tick-based frame timing — measures actual elapsed between ticks :: done
// fps: 1000 / elapsed_ms — reflects compositor responsiveness :: done
// fps_avg: rolling average over HISTORY samples :: done
// fps_1_low: worst 1% frametimes converted to FPS :: done
// ft_history: frametime sparkline history :: done
// Clamped to 5s gap — ignores suspend/resume spikes :: done
// No external deps — pure std timing :: done