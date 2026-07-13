// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/ui/theme.rs
// All colors, gradients, and visual constants for the Soulless launcher.
//
// P1 (config-driven theming): ThemeColors is the runtime theme, loaded once at
// startup from ~/.config/soulless/theme.ron (partial overrides on top of the
// baked-in Chrome look). Default::default() IS the ship look — sharp corners,
// polished steel, ink-on-steel hover, dark blood-red panel.
//
// The legacy `pub const` tokens below still feed the UI until the P2 sweep
// replaces every call site with theme::get().<field>. Until P2 lands,
// theme.ron is loaded but NOT yet visible. Do not remove the consts early.

use cosmic::iced::Color;
use std::sync::OnceLock;

/// The complete runtime theme. Every visual token in one struct.
/// Colors are Copy and the struct is small; get() hands out copies so the
/// storage behind it can change later (live reload) with zero call-site churn.
#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    // ── Canonical text ──
    // ── Canonical text ──
    pub text_steel: Color,
    pub text_ink: Color,
    /// Right-panel headings: drawer names, modal titles. The gold.
    pub drawer_title: Color,
    /// Right-panel secondary lettering: drop hints, empty-state lines.
    pub drawer_hint: Color,
    // ── Window ──
    pub window_bg: Color,
    pub window_corner_radius: f32,
    pub window_border: Color,
    pub window_border_width: f32,
    // ── Steel toolbox panel ──
    pub steel_top: Color,
    pub steel_mid_a: Color,
    pub steel_mid_b: Color,
    pub steel_bottom: Color,
    pub steel_border: Color,
    pub steel_corner_radius: f32,
    pub steel_shadow_color: Color,
    pub steel_text: Color,
    pub steel_vertical_inset: f32,
    // ── Right content panel ──
    pub right_panel_bg: Color,
    pub right_panel_border: Color,
    pub right_panel_corner_radius: f32,
    // ── Monitor widgets ──
    pub widget_bg: Color,
    pub widget_border: Color,
    pub widget_corner_radius: f32,
    pub widget_scale: f32,
    pub widget_height: f32,
    pub widget_spacing: u16,
    // ── Drawer buttons ──
    pub drawer_btn_bg: Color,
    pub drawer_btn_hover: Color,
    pub drawer_btn_active: Color,
    pub drawer_btn_border: Color,
    pub drawer_btn_text: Color,
    pub drawer_btn_text_hover: Color,
}

impl Default for ThemeColors {
    /// THE ship look. Byte-for-byte the values Soulless renders with today.
    fn default() -> Self {
        Self {
            text_steel: Color { r: 0.88, g: 0.89, b: 0.91, a: 1.0 },
            text_ink: Color { r: 0.05, g: 0.05, b: 0.06, a: 1.0 },
            drawer_title: Color { r: 0.85, g: 0.64, b: 0.25, a: 1.0 },
            drawer_hint: Color { r: 0.85, g: 0.64, b: 0.25, a: 0.78 },

            window_bg: Color { r: 0.042, g: 0.042, b: 0.048, a: 1.0 },
            window_corner_radius: 0.0,
            window_border: Color { r: 0.22, g: 0.23, b: 0.25, a: 1.0 },
            window_border_width: 2.0,

            steel_top: Color { r: 0.85, g: 0.87, b: 0.90, a: 1.0 },
            steel_mid_a: Color { r: 0.55, g: 0.58, b: 0.62, a: 1.0 },
            steel_mid_b: Color { r: 0.30, g: 0.32, b: 0.35, a: 1.0 },
            steel_bottom: Color { r: 0.65, g: 0.67, b: 0.70, a: 1.0 },
            steel_border: Color { r: 0.90, g: 0.92, b: 0.95, a: 0.8 },
            steel_corner_radius: 0.0,
            steel_shadow_color: Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 },
            steel_text: Color { r: 0.05, g: 0.05, b: 0.07, a: 1.0 },
            steel_vertical_inset: 24.0,

            right_panel_bg: Color { r: 0.235, g: 0.039, b: 0.039, a: 1.0 },
            right_panel_border: Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 },
            right_panel_corner_radius: 0.0,

            widget_bg: Color { r: 0.118, g: 0.118, b: 0.118, a: 0.9 },
            widget_border: Color { r: 0.3, g: 0.3, b: 0.3, a: 0.4 },
            widget_corner_radius: 0.0,
            widget_scale: 0.75,
            widget_height: 95.0,
            widget_spacing: 4,

            drawer_btn_bg: Color { r: 0.08, g: 0.08, b: 0.09, a: 1.0 },
            drawer_btn_hover: Color { r: 0.75, g: 0.78, b: 0.82, a: 1.0 },
            drawer_btn_active: Color { r: 0.55, g: 0.58, b: 0.62, a: 1.0 },
            drawer_btn_border: Color { r: 0.25, g: 0.26, b: 0.28, a: 0.8 },
            drawer_btn_text: Color { r: 0.88, g: 0.89, b: 0.91, a: 1.0 },
            drawer_btn_text_hover: Color { r: 0.05, g: 0.05, b: 0.06, a: 1.0 },
        }
    }
}

static THEME: OnceLock<ThemeColors> = OnceLock::new();

/// Install the loaded theme. Called exactly once from app init, BEFORE any
/// view code runs. A second call is a no-op (OnceLock).
pub fn init(theme: ThemeColors) {
    let _ = THEME.set(theme);
}

/// The active theme. Falls back to the ship look if init() never ran
/// (tests, tools). Returns a copy on purpose — see struct docs.
pub fn get() -> ThemeColors {
    *THEME.get_or_init(ThemeColors::default)
}

