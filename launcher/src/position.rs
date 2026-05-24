// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

// src/position.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LauncherPosition;

impl LauncherPosition {
    pub const WINDOW_WIDTH: f32 = 932.0;
    pub const WINDOW_HEIGHT: f32 = 720.0;

    // Fixed fallback centered-ish position for now.
    // Real monitor-aware centering can be added once window/screen geometry is wired up.
    pub const CENTER_X: f32 = 494.0;
    pub const CENTER_Y: f32 = 180.0;

    pub fn window_size(self) -> cosmic::iced::Size {
        cosmic::iced::Size::new(Self::WINDOW_WIDTH, Self::WINDOW_HEIGHT)
    }

    pub fn window_position(self) -> cosmic::iced::Point {
        cosmic::iced::Point::new(Self::CENTER_X, Self::CENTER_Y)
    }
}

// === YOUR ORIGINAL COMMENTS (preserved exactly) ===
// use std::path::PathBuf; I am not using at moment not sure if I will :: MRV
// real monitor geometry detection (winit/wayland) :: working
// configurable dock position via settings :: working
// Basic Left position with centered vertical placement :: done
// === IN PROGRESS ===
// real monitor geometry detection (winit/wayland) :: in progress
// configurable dock position via settings :: in progress