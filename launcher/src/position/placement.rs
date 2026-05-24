// MIT License - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// src/position/placement.rs
// Determines where on screen the launcher window is placed.
// Runtime screen detection will live here once wired up.

use super::layout::{PANEL_HEIGHT, WINDOW_HEIGHT, WINDOW_WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LauncherPosition;

impl LauncherPosition {
    // Hardcoded fallback for 2560x1440 display.
    // === IN PROGRESS ===
    // real monitor geometry detection (winit/wayland) :: in progress
    // configurable dock position via settings :: in progress
    const SCREEN_WIDTH: f32 = 2560.0;
    const SCREEN_HEIGHT: f32 = 1440.0;

    pub fn window_size(self) -> cosmic::iced::Size {
        cosmic::iced::Size::new(WINDOW_WIDTH, WINDOW_HEIGHT)
    }

    pub fn window_position(self) -> cosmic::iced::Point {
        let x = (Self::SCREEN_WIDTH - WINDOW_WIDTH) / 2.0;
        let y = Self::SCREEN_HEIGHT - WINDOW_HEIGHT - PANEL_HEIGHT;
        cosmic::iced::Point::new(x, y)
    }
}
