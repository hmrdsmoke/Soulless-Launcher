// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// src/position/placement.rs
// Owns the full lifecycle of the launcher window:
//   - open()  → creates the layer shell surface anchored bottom-right
//   - close() → destroys the surface and exits
//   - focus() → focuses the search bar
// main.rs calls these; placement.rs owns the how and where.

use super::layout::{PANEL_HEIGHT, WINDOW_HEIGHT, WINDOW_WIDTH};
use cosmic::iced::advanced::layout::Limits;
use cosmic::iced::platform_specific::shell::commands::layer_surface::{
    Anchor, KeyboardInteractivity, Layer, get_layer_surface,
};
use cosmic::iced::platform_specific::runtime::wayland::layer_surface::SctkLayerSurfaceSettings;
use cosmic::iced::window;
use cosmic::iced::Task;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LauncherPosition;

#[allow(dead_code)] // TODO: layer shell — see issue #15
impl LauncherPosition {
    // Fallback screen size — replaced by runtime detection in future.
    // === IN PROGRESS ===
    // real monitor geometry detection (winit/wayland) :: in progress
    // configurable dock position via settings :: in progress
    const SCREEN_WIDTH: f32 = 2560.0;
    const SCREEN_HEIGHT: f32 = 1440.0;

    /// Fallback: used only when layer shell is unavailable.
    pub fn window_size(self) -> cosmic::iced::Size {
        cosmic::iced::Size::new(WINDOW_WIDTH, WINDOW_HEIGHT)
    }

    /// Fallback: used only when layer shell is unavailable.
    pub fn window_position(self) -> cosmic::iced::Point {
        let x = Self::SCREEN_WIDTH - WINDOW_WIDTH;
        let y = Self::SCREEN_HEIGHT - WINDOW_HEIGHT - PANEL_HEIGHT;
        cosmic::iced::Point::new(x, y)
    }

    /// Read COSMIC panel's anchor edge from config (plain one-word file:
    /// "Top"/"Bottom"/"Left"/"Right"). Falls back to Bottom if unreadable.
    /// This makes the launcher follow the panel to whichever screen edge it's on.
    fn panel_anchor() -> Anchor {
        match Self::read_panel_str("anchor").trim() {
            "Top" => Anchor::TOP,
            "Left" => Anchor::LEFT,
            "Right" => Anchor::RIGHT,
            _ => Anchor::BOTTOM, // "Bottom" or unknown
        }
    }

    /// Read a single COSMIC panel config value (plain one-word files).
    fn read_panel_str(key: &str) -> String {
        dirs::config_dir()
            .map(|p| p.join(format!("cosmic/com.system76.CosmicPanel.Panel/v1/{key}")))
            .and_then(|p| std::fs::read_to_string(p).ok())
            .unwrap_or_default()
    }

    /// Panel thickness in px, derived from the COSMIC size enum (XS..XL).
    /// Used as a margin so the launcher clears the panel instead of overlapping.
    fn panel_size_px() -> i32 {
        // Complete clearance per panel size (visual height incl. border/spacing).
        // Tuned by observation: bigger panels need proportionally more, smaller less.
        let raw = Self::read_panel_str("size");
        match raw.trim() {
            "XS" => 32,
            "S" => 40,
            "M" => 52,
            "L" => 64,
            "XL" => 78,
            other => other
                .strip_prefix("Custom(")
                .and_then(|s| s.strip_suffix(")"))
                .and_then(|s| s.trim().parse::<i32>().ok())
                .map(|s| s + 12) // custom px + small border allowance
                .unwrap_or(44),
        }
    }

    /// Build the layer shell settings for this launcher.
    fn surface_settings(id: window::Id, screen: Option<(u32, u32)>) -> SctkLayerSurfaceSettings {
        // Screen-aware sizing: the launcher now knows the monitor dimensions (from
        // captured Output events) so it can request a concrete size that FITS the
        // screen, instead of guessing. Theory: the resize loop was the surface
        // unable to reconcile an unknown-relative size with the compositor.
        let (sw, sh) = screen.unwrap_or((1920, 1080));
        let w = (WINDOW_WIDTH as u32).min(sw);
        let h = (WINDOW_HEIGHT as u32).min(sh);

        let mut surface = SctkLayerSurfaceSettings::default();
        surface.id = id;
        surface.keyboard_interactivity = KeyboardInteractivity::OnDemand;
        surface.layer = Layer::Top;
        // Panel-aware: anchor to whichever edge the COSMIC panel is on.
        // Panel sets exclusive_zone=true so the compositor reserves its space;
        // our exclusive_zone=-1 respects that reservation, so the compositor
        // automatically offsets us past the panel — no manual margin needed.
        let anchor = Self::panel_anchor();
        let gap = Self::panel_size_px();
        surface.anchor = anchor;
        // Clear the panel: offset on whichever edge it's anchored to.
        if anchor == Anchor::TOP {
            surface.margin.top = gap;
        } else if anchor == Anchor::LEFT {
            surface.margin.left = gap;
        } else if anchor == Anchor::RIGHT {
            surface.margin.right = gap;
        } else {
            surface.margin.bottom = gap;
        }
        // size: None so autosize controls sizing AND acks the compositor's configure
        // events (the ack completes the layer-shell handshake; without it the
        // compositor re-sends configure forever = the RequestResize flood).
        surface.size = Some((Some(w), Some(h)));
        // Limits bounded by the actual screen, not pinned to the surface size.
        surface.size_limits = Limits::NONE
            .min_width(1.0)
            .min_height(1.0)
            .max_width(sw as f32)
            .max_height(sh as f32);
        surface.exclusive_zone = -1;
        surface.namespace = "soulless-launcher".to_string();
        surface
    }

    /// Open the launcher: create the layer shell surface.
    /// Call this from the subscription after the event loop is running.
    pub fn open<M>(id: window::Id, screen: Option<(u32, u32)>, on_open: impl Fn(window::Id) -> M + Send + 'static) -> Task<M>
    where
        M: Send + 'static,
    {
        get_layer_surface(Self::surface_settings(id, screen)).map(on_open)
    }

    /// Focus the search bar. Call after open() completes.
    pub fn focus_search<M: 'static>() -> Task<M> {
        cosmic::widget::text_input::focus(
            cosmic::widget::Id::new("soulless-search-bar")
        )
    }
}
