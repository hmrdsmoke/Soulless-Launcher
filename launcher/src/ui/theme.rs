// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/ui/theme.rs
// All colors, gradients, and visual constants for the Soulless launcher.

use cosmic::iced::Color;

// ── Canonical text (launcher-wide) ───────────────────────────────────────────
// THE text pair. Every label uses TEXT_STEEL at rest and inverts to TEXT_INK
// when its row/button is hovered or keyboard-focused (hover feeds focus via
// FocusApp, so one condition covers both). Tune the steel here; the whole
// launcher follows.
/// Polished-steel text on dark surfaces — the default for all labels.
pub const TEXT_STEEL: Color = Color { r: 0.88, g: 0.89, b: 0.91, a: 1.0 };
/// Ink — text color when the item under it is hovered/focused (on steel).
pub const TEXT_INK: Color = Color { r: 0.05, g: 0.05, b: 0.06, a: 1.0 };

// ── Window ───────────────────────────────────────────────────────────────────
pub const WINDOW_BG: Color = Color { r: 0.042, g: 0.042, b: 0.048, a: 1.0 };
pub const WINDOW_CORNER_RADIUS: f32 = 0.0;
pub const WINDOW_BORDER: Color = Color { r: 0.22, g: 0.23, b: 0.25, a: 1.0 };
pub const WINDOW_BORDER_WIDTH: f32 = 2.0;

// ── Steel toolbox panel ───────────────────────────────────────────────────────
// Polished silver/chrome — bright highlights, dark valleys, high contrast
pub const STEEL_TOP: Color    = Color { r: 0.85, g: 0.87, b: 0.90, a: 1.0 }; // bright highlight
pub const STEEL_MID_A: Color  = Color { r: 0.55, g: 0.58, b: 0.62, a: 1.0 }; // mid silver
pub const STEEL_MID_B: Color  = Color { r: 0.30, g: 0.32, b: 0.35, a: 1.0 }; // dark valley
pub const STEEL_BOTTOM: Color = Color { r: 0.65, g: 0.67, b: 0.70, a: 1.0 }; // reflected light
pub const STEEL_BORDER: Color = Color { r: 0.90, g: 0.92, b: 0.95, a: 0.8 }; // bright edge
pub const STEEL_CORNER_RADIUS: f32 = 0.0;
pub const STEEL_SHADOW_COLOR: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 };
pub const STEEL_TEXT: Color = Color { r: 0.05, g: 0.05, b: 0.07, a: 1.0 }; // dark text on silver
/// How much shorter the steel panel is vs window height
pub const STEEL_VERTICAL_INSET: f32 = 24.0;

// ── Right content panel ───────────────────────────────────────────────────────
pub const RIGHT_PANEL_BG: Color = Color { r: 0.235, g: 0.039, b: 0.039, a: 1.0 };
pub const RIGHT_PANEL_BORDER: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
pub const RIGHT_PANEL_CORNER_RADIUS: f32 = 0.0;

// ── Monitor widgets ───────────────────────────────────────────────────────────
pub const WIDGET_BG: Color = Color { r: 0.118, g: 0.118, b: 0.118, a: 0.9 };
pub const WIDGET_BORDER: Color = Color { r: 0.3, g: 0.3, b: 0.3, a: 0.4 };
pub const WIDGET_CORNER_RADIUS: f32 = 0.0;
/// Scale factor for monitor widgets (0.75 = 25% smaller)
#[allow(dead_code)]
pub const WIDGET_SCALE: f32 = 0.75;

// ── Drawer button colors ──────────────────────────────────────────────────────
pub const DRAWER_BTN_BG: Color       = Color { r: 0.08, g: 0.08, b: 0.09, a: 1.0 }; // metallic black
pub const DRAWER_BTN_HOVER: Color    = Color { r: 0.75, g: 0.78, b: 0.82, a: 1.0 }; // steel highlight
pub const DRAWER_BTN_ACTIVE: Color   = Color { r: 0.55, g: 0.58, b: 0.62, a: 1.0 }; // mid steel
pub const DRAWER_BTN_BORDER: Color   = Color { r: 0.25, g: 0.26, b: 0.28, a: 0.8 }; // dark edge
pub const DRAWER_BTN_TEXT: Color     = TEXT_STEEL; // canonical steel
pub const DRAWER_BTN_TEXT_HOVER: Color = TEXT_INK; // canonical ink

/// Height of each monitor widget in the grid
pub const WIDGET_HEIGHT: f32 = 95.0;
/// Spacing between monitor widgets in the grid
pub const WIDGET_SPACING: u16 = 4;
