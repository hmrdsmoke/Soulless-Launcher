// GPL-3.0-or-later - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

pub mod graph;
pub mod stats;
pub mod view;

use cosmic::iced::{Element, Subscription};
use std::time::Duration;

// ── Constants ─────────────────────────────────────────────────────────────────

/// Number of samples kept in the rolling graph history.
pub const HISTORY: usize = 30;

/// Stats polling interval.
const TICK_MS: u64 = 1_000;

// ── Message ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Message {
    StatsTick,
}

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SystemState {
    pub stats: stats::StatsState,
}

impl SystemState {
    pub fn new() -> Self {
        Self {
            stats: stats::StatsState::new(),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::StatsTick => {
                self.stats.tick();
            }
        }
    }
}

impl Default for SystemState {
    fn default() -> Self {
        Self::new()
    }
}

// ── Public view entry-point ───────────────────────────────────────────────────

pub fn view(state: &SystemState) -> Element<'_, Message> {
    view::view(state)
}

// ── Subscription ─────────────────────────────────────────────────────────────

pub fn subscription() -> Subscription<Message> {
    cosmic::iced::time::every(Duration::from_millis(TICK_MS))
        .map(|_| Message::StatsTick)
}

// === DONE ===
// SystemState: composes StatsState :: done
// Message: StatsTick :: done
// update(): dispatches to stats.tick() :: done
// subscription(): 1s tick :: done
// view(): delegates to view::view() :: done
// HISTORY constant shared with stats.rs :: done