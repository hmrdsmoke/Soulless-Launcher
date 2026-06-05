// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.



mod config;
mod drawers;
mod ui;
mod network_monitor;
mod system_monitor;
mod hardware_monitor;
mod fps_monitor;
mod keybinds;
mod position;
mod search;
mod vault;
pub mod registry;
use cosmic::iced::window;
use crate::position::LauncherPosition;

mod app;
mod utils;
mod easter_egg;



// ── Entry point ──────────────────────────────────────────────────────────────

fn main() -> cosmic::iced::Result {
    if !position::ensure_single_instance() {
        eprintln!("Soulless is already running.");
        return Ok(());
    }

    let position = LauncherPosition;

    cosmic::iced::application(
        app::Soulless::new,
        app::Soulless::update,
        app::Soulless::view,
    )
    .subscription(app::Soulless::subscription)
    .theme(app::Soulless::theme)
    .window_size(position.window_size())
    .position(window::Position::Specific(
        position.window_position(),
    ))
    .decorations(false)
    .transparent(true)
    .resizable(false)
    .run()
}

