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

/// Which COSMIC bar wing the applet is in (drives launcher corner anchoring).
#[derive(Clone, Copy)]
enum Wing {
    Left,
    Right,
    Center,
}

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

    /// Read a single config value from a given bar ("Panel" or "Dock").
    fn read_bar_str(bar: &str, key: &str) -> String {
        dirs::config_dir()
            .map(|p| p.join(format!("cosmic/com.system76.CosmicPanel.{bar}/v1/{key}")))
            .and_then(|p| std::fs::read_to_string(p).ok())
            .unwrap_or_default()
    }

    const APPLET_ID: &'static str = "com.github.hmrdsmoke.soulless-applet";

    /// Find which bar + wing the soulless applet is in.
    /// Searches Panel first, then Dock. plugins_wings format is
    /// `Some(([left...], [right...]))`; plugins_center is `Some([...])`.
    /// Returns (bar, wing). Defaults to ("Panel", Center) if not found.
    fn find_applet_bar() -> (&'static str, Wing) {
        for bar in ["Panel", "Dock"] {
            let wings = Self::read_bar_str(bar, "plugins_wings");
            // Split the two wing arrays. The file has the left array first,
            // then the right. Find the boundary between `]` and the next `[`.
            if wings.contains(Self::APPLET_ID) {
                // Locate applet position and the array boundary "], ["
                let pos = wings.find(Self::APPLET_ID).unwrap();
                let boundary = wings.find("], [").or_else(|| wings.find("],["));
                let wing = match boundary {
                    Some(b) if pos < b => Wing::Left,
                    Some(_) => Wing::Right,
                    None => Wing::Left, // single array, treat as left
                };
                return (bar, wing);
            }
            let center = Self::read_bar_str(bar, "plugins_center");
            if center.contains(Self::APPLET_ID) {
                return (bar, Wing::Center);
            }
        }
        ("Panel", Wing::Center)
    }

    /// Bar thickness in px from the COSMIC size enum (XS..XL or Custom).
    fn bar_size_px(bar: &str) -> i32 {
        match Self::read_bar_str(bar, "size").trim() {
            "XS" => 32,
            "S" => 40,
            "M" => 52,
            "L" => 64,
            "XL" => 78,
            other => other
                .strip_prefix("Custom(")
                .and_then(|s| s.strip_suffix(")"))
                .and_then(|s| s.trim().parse::<i32>().ok())
                .map(|s| s + 12)
                .unwrap_or(44),
        }
    }

    /// Read a bar's edge anchor.
    fn bar_anchor(bar: &str) -> Anchor {
        match Self::read_bar_str(bar, "anchor").trim() {
            "Top" => Anchor::TOP,
            "Left" => Anchor::LEFT,
            "Right" => Anchor::RIGHT,
            _ => Anchor::BOTTOM,
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
        // Applet-aware placement: find which bar (Panel/Dock) + wing the applet
        // is in, anchor the launcher to that bar's edge AND that wing's side so
        // it pops up from the corner where the button lives. Clear the bar with
        // its size-derived margin.
        let (bar, wing) = Self::find_applet_bar();
        let edge = Self::bar_anchor(bar);
        let gap = Self::bar_size_px(bar);

        // Edge flag = the bar's screen edge. Wing flag = left/right hug.
        let mut anchor = edge;
        match wing {
            Wing::Left => anchor = anchor | Anchor::LEFT,
            Wing::Right => anchor = anchor | Anchor::RIGHT,
            Wing::Center => {} // no horizontal flag -> compositor centers
        }
        surface.anchor = anchor;

        // Clear the bar on whichever edge it occupies.
        if edge == Anchor::TOP {
            surface.margin.top = gap;
        } else if edge == Anchor::LEFT {
            surface.margin.left = gap;
        } else if edge == Anchor::RIGHT {
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
