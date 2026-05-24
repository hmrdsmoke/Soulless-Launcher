// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

pub mod fps;
pub mod graph;
pub mod view;

use cosmic::iced::{Element, Subscription};
use std::time::Duration;

// ── Constants ─────────────────────────────────────────────────────────────────

/// Number of frametime samples kept for the sparkline.
pub const HISTORY: usize = 60;

/// How often we re-read the MangoHud log file.
const TICK_FPS_MS: u64 = 500;

// ── Message ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Message {
    FpsTick,
}

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FpsMonitorState {
    pub fps: fps::FpsState,
}

impl FpsMonitorState {
    pub fn new() -> Self {
        Self {
            fps: fps::FpsState::new(),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::FpsTick => {
                self.fps.tick();
            }
        }
    }
}

impl Default for FpsMonitorState {
    fn default() -> Self {
        Self::new()
    }
}

// ── Public view entry-point ───────────────────────────────────────────────────

pub fn view(state: &FpsMonitorState) -> Element<'_, Message> {
    view::view(state)
}

// ── Subscription ─────────────────────────────────────────────────────────────

pub fn subscription() -> Subscription<Message> {
    cosmic::iced::time::every(Duration::from_millis(TICK_FPS_MS))
        .map(|_| Message::FpsTick)
}

// === DONE ===
// FpsMonitorState: wraps FpsState :: done
// Message: FpsTick :: done
// update(): dispatches tick :: done
// subscription(): 500ms poll :: done
// view(): delegates to view::view() :: done
// HISTORY constant shared with fps.rs :: done