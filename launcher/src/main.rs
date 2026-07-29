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
mod terminal;
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

    // ── CLI: `soulless-launcher toggle` ──────────────────────────────────
    // No clap — one subcommand, matched directly. run_single_instance does the
    // routing: if the daemon owns the D-Bus name, this process sends
    // ActivateAction("toggle") to it and exits without starting a second
    // instance. If no daemon is running, it becomes the daemon.
    let flags = match std::env::args().nth(1).as_deref() {
        None => app::SoullessFlags::default(),
        Some("toggle") => app::SoullessFlags {
            subcommand: Some(app::SoullessSubCommand::Toggle),
        },
        Some("-h") | Some("--help") | Some("help") => {
            print_help();
            return Ok(());
        }
        Some("-V") | Some("--version") => {
            println!("soulless-launcher {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some(other) => {
            // Unrecognized args must NOT fall through to the daemon: without
            // this, `soulless-launcher --help` would start a second instance
            // instead of printing help.
            eprintln!("soulless-launcher: unrecognized argument '{other}'");
            eprintln!("Try 'soulless-launcher --help' for more information.");
            std::process::exit(1);
        }
    };

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
    cosmic::app::run_single_instance::<app::Soulless>(settings, flags)
}

/// Usage text. Goes to stdout and exits 0, per convention.
fn print_help() {
    println!("Usage: soulless-launcher [COMMAND]");
    println!();
    println!("A command-center launcher for the COSMIC desktop: universal search,");
    println!("custom drawers, an encrypted vault, and live system monitors.");
    println!();
    println!("Commands:");
    println!("  toggle           Show the launcher if hidden, hide it if visible");
    println!();
    println!("Options:");
    println!("  -h, --help       Print this help and exit");
    println!("  -V, --version    Print version information and exit");
    println!();
    println!("With no arguments, soulless-launcher runs as the resident daemon. It is");
    println!("normally started at login from /etc/xdg/autostart and stays warm, showing");
    println!("and hiding its layer surface on request rather than starting fresh.");
    println!();
    println!("See soulless-launcher(1) for full documentation.");
}
