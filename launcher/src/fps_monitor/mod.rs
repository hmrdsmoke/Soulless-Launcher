// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/fps_monitor/mod.rs
// FPS monitor module: state, subscription, and constants.

pub mod fps;
pub mod graph;
pub mod monitors;
pub mod view;

use cosmic::iced::{Element, Subscription};
use cosmic::iced::event::wayland::OutputEvent;

// ── Constants ─────────────────────────────────────────────────────────────────

/// Number of frametime samples kept for the sparkline.
pub const HISTORY: usize = 60;

// ── Message ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Message {
    FpsTick,
    Output(OutputEvent, String),
    /// The launcher surface was configured at this logical size —
    /// road-B join to the census (app.rs forwards its configure event).
    SurfaceOn(i32, i32),
}

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FpsMonitorState {
    pub fps: fps::FpsState,
    pub monitors: monitors::MonitorsState,
}

impl FpsMonitorState {
    pub fn new() -> Self {
        Self {
            fps: fps::FpsState::new(),
            monitors: monitors::MonitorsState::default(),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::FpsTick => {
                self.fps.tick();
                self.monitors.record_fps(self.fps.fps);
            }
            Message::Output(evt, key) => {
                self.monitors.apply(evt, key);
            }
            Message::SurfaceOn(w, h) => {
                self.monitors.surface_on(w, h);
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
    // Real frame callbacks, not a fixed timer. wayland_frames() fires on the
    // compositor's wl_surface.frame callback (and iced's RedrawRequested),
    // so the interval between ticks IS the true frame interval — and fps.rs,
    // which just times the gap between ticks and divides, now reports actual
    // frames rather than "did the event loop service a 16ms timer."
    //
    // Reactive consequence: iced only paints when the surface has a reason to,
    // so at full idle this fires rarely and the readout drops toward zero. It
    // climbs to the real refresh rate during interaction and while the monitor
    // graphs are repainting. That's honest — it measures frames actually drawn.
    cosmic::iced::window::wayland_frames().map(|_| Message::FpsTick)
}

/// Monitor census subscription — kept OUT of the visibility-gated batch.
/// Output Created events fire once at registry bind; a subscriber created
/// on first surface-open starts deaf. app.rs runs this from tick zero.
pub fn monitors_subscription() -> Subscription<Message> {
    monitors::subscription().map(|(evt, key)| Message::Output(evt, key))
}

// === DONE ===
// FpsMonitorState: wraps FpsState :: done
// Message: FpsTick :: done
// update(): dispatches tick :: done
// subscription(): frame ticks (gated); monitors_subscription(): census, ungated via app.rs :: done
// view(): delegates to view::view() :: done
// HISTORY constant shared with fps.rs :: done
// monitors.rs: census + size-join active output + live fps fan-in :: done (concern three)