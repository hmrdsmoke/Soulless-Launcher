// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/hardware_monitor/mod.rs
// Hardware monitor module: state, subscription, and constants.

pub mod graph;
pub mod hardware;
pub mod view;

use cosmic::iced::{Element, Subscription};
use std::time::Duration;

// ── Constants ─────────────────────────────────────────────────────────────────

/// Number of samples kept in rolling graph histories.
pub const HISTORY: usize = 30;

/// Hardware poll interval — 2 s keeps sysfs/NVML overhead minimal.
const TICK_HW_MS: u64 = 2_000;

// ── Message ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Message {
    HardwareTick,
}

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct HardwareMonitorState {
    pub hw: hardware::HardwareState,
}

impl HardwareMonitorState {
    pub fn new() -> Self {
        Self {
            hw: hardware::HardwareState::new(),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::HardwareTick => {
                self.hw.tick();
            }
        }
    }
}

impl Default for HardwareMonitorState {
    fn default() -> Self {
        Self::new()
    }
}

// ── Public view entry-point ───────────────────────────────────────────────────

pub fn view(state: &HardwareMonitorState) -> Element<'_, Message> {
    view::view(state)
}

// ── Subscription ─────────────────────────────────────────────────────────────

pub fn subscription() -> Subscription<Message> {
    cosmic::iced::time::every(Duration::from_millis(TICK_HW_MS))
        .map(|_| Message::HardwareTick)
}

// === DONE ===
// HardwareMonitorState: wraps HardwareState :: done
// Message: HardwareTick :: done
// update(): dispatches tick :: done
// subscription(): 2s hardware poll :: done
// view(): delegates to view::view() :: done
// HISTORY constant shared with hardware.rs :: done