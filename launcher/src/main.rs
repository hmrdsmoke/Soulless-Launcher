// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI) and Claude (Anthropic).
// Do not remove these comments.
// launcher/src/main.rs
// Launcher entry point - module wiring and application startup.

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
    // Initialize tracing so cosmic/iced/zbus internal logs are visible.
    // Controlled by RUST_LOG (defaults to warn). Without this, cosmic's own
    // warn!/info! (e.g. single-instance activation messages) are invisible.
    {
        use tracing_subscriber::fmt;
        use tracing_subscriber::EnvFilter;
        let _ = fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
            )
            .try_init();
    }

    // NOTE: removed the custom ensure_single_instance() flock check. cosmic's
    // run_single_instance now provides single-instance behavior via D-Bus. The
    // lockfile actively BREAKS that model: run_single_instance expects a second
    // process to start, detect the running daemon over D-Bus, send an activation,
    // and exit — but the exclusive flock would block that second process from ever
    // reaching run_single_instance.

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
