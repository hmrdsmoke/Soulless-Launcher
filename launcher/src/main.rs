// GPL-3.0-or-later - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI) and Claude (Anthropic).
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

mod app;
mod keep_alive;
mod utils;
mod easter_egg;

// ── Entry point ──────────────────────────────────────────────────────────────

fn main() -> cosmic::iced::Result {
    if !position::ensure_single_instance() {
        eprintln!("Soulless is already running.");
        return Ok(());
    }

    let settings = cosmic::app::Settings::default()
        .size(cosmic::iced::Size::new(
            crate::position::layout::WINDOW_WIDTH,
            crate::position::layout::WINDOW_HEIGHT,
        ))
        .client_decorations(false)
        .transparent(true)
        .resizable(None)
        .no_main_window(true)
        .exit_on_close(false);

    cosmic::app::run_single_instance::<app::Soulless>(settings, app::SoullessFlags::default())
}