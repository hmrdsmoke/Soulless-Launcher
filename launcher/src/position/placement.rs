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

use super::layout::{WINDOW_HEIGHT, WINDOW_WIDTH};
use cosmic::iced::advanced::layout::Limits;
use cosmic::iced::platform_specific::shell::commands::layer_surface::{
    Anchor, KeyboardInteractivity, Layer, destroy_layer_surface, get_layer_surface,
};
use cosmic::iced::platform_specific::runtime::wayland::layer_surface::SctkLayerSurfaceSettings;
use cosmic::iced::window;
use cosmic::iced::Task;

/// Which array of a COSMIC bar's wings the applet is in.
/// First/Second are orientation-neutral: for a horizontal bar (top/bottom)
/// First=left, Second=right; for a vertical bar (left/right) First=top,
/// Second=bottom. The mapping to a screen direction happens at anchor time.
#[derive(Clone, Copy)]
enum Wing {
    First,
    Second,
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LauncherPosition;

impl LauncherPosition {
    // Fallback screen size — replaced by runtime detection in future.
    // === IN PROGRESS ===
    // real monitor geometry detection (winit/wayland) :: in progress
    // configurable dock position via settings :: in progress

    /// Fallback: used only when layer shell is unavailable.

    /// Fallback: used only when layer shell is unavailable.

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
                    Some(b) if pos < b => Wing::First,
                    Some(_) => Wing::Second,
                    None => Wing::First, // single array, treat as first
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
    fn surface_settings(id: window::Id, _screen: Option<(u32, u32)>) -> SctkLayerSurfaceSettings {
        // Match cosmic-launcher show() EXACTLY: Exclusive keyboard, TOP anchor,
        // size None (autosize), max_width 600. Proven-working real surface.
        // Applet-corner positioning removed temporarily to get a working baseline;
        // re-add once the window reliably shows.
        let mut surface = SctkLayerSurfaceSettings::default();
        surface.id = id;
        surface.keyboard_interactivity = KeyboardInteractivity::Exclusive;
        surface.layer = Layer::Top;
        surface.anchor = Anchor::all();
        surface.namespace = "launcher".to_string();
        surface.size = Some((None, None));
        surface.size_limits = Limits::NONE.min_width(1.0).min_height(1.0).max_width(600.0);
        surface.exclusive_zone = -1;
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


    /// Create a DUMMY bottom-layer surface at init to anchor the launcher onto
    /// the Wayland connection — especially the inherited host socket from
    /// X-HostWaylandDisplay=true (WAYLAND_SOCKET). Mirrors cosmic-launcher's
    /// create_dummy_layer_surface: without anchoring the connection at startup,
    /// the real surface (open()) fails to show on the inherited socket.
    /// Bottom layer + None keyboard + empty input zone = invisible, inert, and
    /// will not trigger the RequestResize flood the real surface did at init.
    pub fn create_dummy<M>(
        id: window::Id,
        on_open: impl Fn(window::Id) -> M + Send + 'static,
    ) -> Task<M>
    where
        M: Send + 'static,
    {
        let mut surface = SctkLayerSurfaceSettings::default();
        surface.id = id;
        surface.layer = Layer::Bottom;
        surface.keyboard_interactivity = KeyboardInteractivity::None;
        surface.input_zone = Some(Vec::new());
        surface.anchor = Anchor::all();
        surface.namespace = "soulless_launcher_dummy".to_string();
        surface.size = Some((Some(600), Some(200)));
        surface.exclusive_zone = -1;
        surface.size_limits = Limits::NONE;
        get_layer_surface(surface).map(on_open)
    }

    /// Destroys the layer shell surface. Mirrors open() so all surface
    /// lifecycle stays in this module (per the file's open/close contract).
    pub fn close<M>(id: window::Id) -> Task<M>
    where
        M: Send + 'static,
    {
        destroy_layer_surface(id)
    }
}
