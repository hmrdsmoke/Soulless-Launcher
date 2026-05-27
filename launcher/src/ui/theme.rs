// MIT License - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// src/ui/theme.rs
// All colors, gradients, and visual constants for the Soulless launcher.

use cosmic::iced::Color;

// ── Window ───────────────────────────────────────────────────────────────────
pub const WINDOW_BG: Color = Color { r: 0.06, g: 0.06, b: 0.06, a: 0.98 };
pub const WINDOW_CORNER_RADIUS: f32 = 12.0;

// ── Steel toolbox panel ───────────────────────────────────────────────────────
// Polished silver/chrome — bright highlights, dark valleys, high contrast
pub const STEEL_TOP: Color    = Color { r: 0.85, g: 0.87, b: 0.90, a: 1.0 }; // bright highlight
pub const STEEL_MID_A: Color  = Color { r: 0.55, g: 0.58, b: 0.62, a: 1.0 }; // mid silver
pub const STEEL_MID_B: Color  = Color { r: 0.30, g: 0.32, b: 0.35, a: 1.0 }; // dark valley
pub const STEEL_BOTTOM: Color = Color { r: 0.65, g: 0.67, b: 0.70, a: 1.0 }; // reflected light
pub const STEEL_BORDER: Color = Color { r: 0.90, g: 0.92, b: 0.95, a: 0.8 }; // bright edge
pub const STEEL_CORNER_RADIUS: f32 = 12.0;
pub const STEEL_SHADOW_COLOR: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 };
pub const STEEL_TEXT: Color = Color { r: 0.05, g: 0.05, b: 0.07, a: 1.0 }; // dark text on silver
/// How much shorter the steel panel is vs window height
pub const STEEL_VERTICAL_INSET: f32 = 24.0;

// ── Right content panel ───────────────────────────────────────────────────────
pub const RIGHT_PANEL_BG: Color = Color { r: 0.235, g: 0.039, b: 0.039, a: 1.0 };
pub const RIGHT_PANEL_BORDER: Color = Color { r: 0.706, g: 0.078, b: 0.078, a: 0.3 };
pub const RIGHT_PANEL_CORNER_RADIUS: f32 = 8.0;

// ── Monitor widgets ───────────────────────────────────────────────────────────
pub const WIDGET_BG: Color = Color { r: 0.118, g: 0.118, b: 0.118, a: 0.9 };
pub const WIDGET_BORDER: Color = Color { r: 0.3, g: 0.3, b: 0.3, a: 0.4 };
pub const WIDGET_CORNER_RADIUS: f32 = 8.0;
/// Scale factor for monitor widgets (0.75 = 25% smaller)
pub const WIDGET_SCALE: f32 = 0.75;
