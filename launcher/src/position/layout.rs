// MIT License - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// src/position/layout.rs
// Single source of truth for all window and panel dimensions.
// drawers.rs should import from here instead of hardcoding values.

/// Total window width
pub const WINDOW_WIDTH: f32 = 700.0;

/// Total window height
pub const WINDOW_HEIGHT: f32 = 900.0;

/// Left sidebar (toolbox/drawers panel) width
pub const TOOLBOX_WIDTH: f32 = 220.0;

/// Right content panel width
pub const RIGHT_PANEL_WIDTH: f32 = 460.0;

/// Gap between left and right panels
pub const PANEL_SPACING: f32 = 12.0;

/// Estimated panel/dock height at bottom of screen (used for placement offset)
pub const PANEL_HEIGHT: f32 = 0.0;
