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

    // Single-instance: kernel flock guard (see daemon_lock below), reinstated
    // Aug 20 2026. cosmic's run_single_instance dedups over D-Bus and still
    // provides the winner's activation machinery — but inside flatpak each
    // instance sits behind its own xdg-dbus-proxy and the loser never learns
    // the name is taken: at login the autostart daemon and the applet-spawned
    // daemon BOTH fully initialize, and the loser's full-screen dismiss-catcher
    // surface lives forever with no D-Bus reachability to tear it down —
    // eating every click on the desktop (virgin-box sandbox finding). The old
    // objection (flock blocks the forwarding) is solved: the loser now delivers
    // the activation itself, then exits. The exit is the cure; the forward is
    // courtesy.

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

    // ── Panel-birth guard ────────────────────────────────────────────────
    // cosmic-panel execs every id listed in its applet config (plugins_wings /
    // plugins_center) and stamps COSMIC_PANEL_* into each child's environment.
    // If a user adds the MAIN app id to that list (one misclick in the applet
    // picker), the panel births a raw launcher as a panel child at every
    // login: it wins the flock, squats the D-Bus name, and every applet click
    // then summons a half-alive resident that dismisses on mouse release and
    // leaves a click-eating ghost surface (virgin-box finding, Aug 20 2026).
    // The legitimate applet spawn ALSO inherits the fingerprint — the applet
    // is itself a panel child — so the fingerprint alone cannot convict: the
    // applet blesses its spawn with SOULLESS_SPAWN=applet, and residency is
    // refused only on fingerprint-without-blessing. D-Bus activation,
    // autostart, terminal, and flatpak-run births carry no fingerprint and
    // pass untouched.
    if std::env::var_os("COSMIC_PANEL_NAME").is_some()
        && std::env::var("SOULLESS_SPAWN").as_deref() != Ok("applet")
    {
        eprintln!("[launcher] refusing residency: exec'd as a cosmic-panel child (COSMIC_PANEL_NAME set) without the applet's spawn blessing.");
        eprintln!("[launcher] cure: remove 'com.github.hmrdsmoke.soulless-launcher' from the panel's applet list — only 'com.github.hmrdsmoke.soulless-launcher.Applet' belongs there.");
        return Ok(());
    }

    // Sandbox residency guard. Inside flatpak, layer-shell only works on
    // the privileged socket cosmic-comp hands the applet's spawn; any other
    // sandbox birth -- the shortcut's `flatpak run ... toggle`, the store's
    // Open button, a terminal run -- gets the filtered socket and would
    // become a MUTE daemon: wins the flock, owns the D-Bus name, can never
    // create a surface, and wedges the applet button until pkill (test-box
    // finding, Aug 26 2026). Same disease as the panel-birth ghost, third
    // birth canal. Unblessed sandbox births stay clients: forward if a
    // daemon exists (Lost arm below), refuse residency if none does.
    // Native is untouched: autostart and terminal runs there are fully
    // capable daemons by design.
    let sandbox_unblessed = std::path::Path::new("/.flatpak-info").exists()
        && std::env::var("SOULLESS_SPAWN").as_deref() != Ok("applet");

    // Winner holds the flock for the whole process lifetime; the kernel
    // releases it on any exit, clean or not. Loser forwards and dies before
    // creating a single surface.
    let _daemon_lock = match daemon_lock() {
        DaemonLock::Won(file) => {
            if sandbox_unblessed {
                drop(file); // free the flock for the next blessed birth
                eprintln!("[launcher] refusing residency: sandbox birth without the applet's spawn blessing -- this socket cannot create layer surfaces.");
                eprintln!("[launcher] the daemon starts from the panel applet; click the Soulless applet once (or re-log) and the shortcut will work.");
                return Ok(());
            }
            Some(file)
        }
        DaemonLock::Lost => {
            // Warm-up twin (applet init-birth): daemon already exists, and
            // nobody clicked -- exit silently instead of forwarding a
            // visible open.
            if std::env::var_os("SOULLESS_WARM").is_some() {
                return Ok(());
            }
            forward_activation(matches!(
                flags.subcommand,
                Some(app::SoullessSubCommand::Toggle)
            ));
            return Ok(());
        }
        DaemonLock::Unavailable => {
            if sandbox_unblessed {
                eprintln!("[launcher] refusing residency: sandbox birth without the applet's spawn blessing (lock unavailable).");
                return Ok(());
            }
            None
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

enum DaemonLock {
    /// We hold the lock — we are the daemon.
    Won(std::fs::File),
    /// Another live process holds it — forward and exit.
    Lost,
    /// Couldn't even try (no runtime dir, fs error). Availability beats
    /// dedup: proceed as daemon rather than refuse to start.
    Unavailable,
}

/// Exclusive, non-blocking flock on a file every instance can see.
/// Natively that's $XDG_RUNTIME_DIR; in flatpak it's the per-app dir
/// $XDG_RUNTIME_DIR/app/$FLATPAK_ID — the same host directory bind-mounted
/// into every sandbox instance of this app id, which is exactly why the
/// kernel can referee a race the proxied bus cannot.
fn daemon_lock() -> DaemonLock {
    use std::os::fd::AsRawFd;
    let dir = match std::env::var_os("FLATPAK_ID") {
        Some(id) => std::env::var_os("XDG_RUNTIME_DIR")
            .map(|r| std::path::PathBuf::from(r).join("app").join(id)),
        None => std::env::var_os("XDG_RUNTIME_DIR").map(std::path::PathBuf::from),
    };
    let Some(dir) = dir else { return DaemonLock::Unavailable };
    let _ = std::fs::create_dir_all(&dir);
    let Ok(file) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(dir.join("soulless-launcher.lock"))
    else {
        return DaemonLock::Unavailable;
    };
    match unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } {
        0 => DaemonLock::Won(file),
        _ => DaemonLock::Lost,
    }
}

/// Loser path: deliver the activation run_single_instance would have
/// forwarded, then let main return. Retries briefly because at login the
/// winner may hold the flock before it owns the D-Bus name.
fn forward_activation(toggle: bool) {
    let Ok(conn) = zbus::blocking::Connection::session() else { return };
    let dest = "com.github.hmrdsmoke.SoullessLauncher";
    let path = "/com/github/hmrdsmoke/SoullessLauncher";
    let iface = "org.freedesktop.DbusActivation";
    let platform: std::collections::HashMap<String, zbus::zvariant::Value> =
        std::collections::HashMap::new();
    for _ in 0..30 {
        let res = if toggle {
            conn.call_method(
                Some(dest), path, Some(iface), "ActivateAction",
                &("toggle", Vec::<String>::new(), platform.clone()),
            )
        } else {
            conn.call_method(Some(dest), path, Some(iface), "Activate", &(platform.clone(),))
        };
        if res.is_ok() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    eprintln!("[soulless] lock held but activation undeliverable — exiting anyway");
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
