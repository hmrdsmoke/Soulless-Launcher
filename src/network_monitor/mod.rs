// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

pub mod bandwidth;
pub mod graph;
pub mod ping;
pub mod view;

use cosmic::iced::{Element, Subscription};
use std::time::Duration;

// ── Constants ─────────────────────────────────────────────────────────────────

/// Number of samples kept in the rolling graph history.
pub const HISTORY: usize = 30;

/// Bandwidth polling interval.
const TICK_BW_MS:   u64 = 1_000;

/// Ping polling interval.
const TICK_PING_S:  u64 = 10;

// ── Message ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Message {
    BandwidthTick,
    PingResult { ping_ms: f32, jitter_ms: f32 },
}

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct NetworkState {
    pub bandwidth: bandwidth::BandwidthState,
    pub ping:      ping::PingState,
}

impl NetworkState {
    pub fn new() -> Self {
        Self {
            bandwidth: bandwidth::BandwidthState::new(),
            ping:      ping::PingState::new(),
        }
    }

    /// Dispatch an incoming message to the correct substate.
    pub fn update(&mut self, message: Message) {
        match message {
            Message::BandwidthTick => {
                self.bandwidth.tick();
            }

            Message::PingResult { ping_ms, jitter_ms } => {
                self.ping.update(ping_ms, jitter_ms);
            }
        }
    }
}

impl Default for NetworkState {
    fn default() -> Self {
        Self::new()
    }
}

// ── Public view entry-point ───────────────────────────────────────────────────

pub fn view(state: &NetworkState) -> Element<'_, Message> {
    view::view(state)
}

// ── Subscription ─────────────────────────────────────────────────────────────

pub fn subscription() -> Subscription<Message> {
    Subscription::batch([
        // 1 s bandwidth tick
        cosmic::iced::time::every(Duration::from_millis(TICK_BW_MS))
            .map(|_| Message::BandwidthTick),

        // 10 s ping — measure() is synchronous so runs on the subscription
        // worker thread, keeping the UI responsive.
        cosmic::iced::time::every(Duration::from_secs(TICK_PING_S))
            .map(|_| {
                let (ping_ms, jitter_ms) = ping::measure();
                Message::PingResult { ping_ms, jitter_ms }
            }),
    ])
}

// === DONE ===
// NetworkState: composes BandwidthState + PingState :: done
// Message: BandwidthTick + PingResult :: done
// update(): dispatches to correct substate :: done
// subscription(): 1s bandwidth tick + 10s ping tick :: done
// view(): delegates to view::view() :: done
// HISTORY constant shared with bandwidth.rs :: done