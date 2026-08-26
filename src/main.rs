// SPDX-License-Identifier: GPL-3.0-or-later
// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// applet/src/main.rs
// Applet entry point - initializes localization and runs the applet.

mod app;
mod config;
mod i18n;

/// The panel's privileged Wayland socket, claimed at startup.
///
/// cosmic-comp filters privileged protocols (zwlr_layer_shell_v1 among 21
/// others) from sandboxed clients — see docs/sandbox-withheld-protocols.txt in
/// the launcher repo. The ONE exemption is a client bearing the
/// `com.system76.CosmicPanel` security-context, which cosmic-panel mints for
/// applets whose desktop entry declares `X-HostWaylandDisplay=true` and hands
/// over as X_PRIVILEGED_WAYLAND_SOCKET.
///
/// We take that fd for the LAUNCHER: it is the only route by which a
/// Flatpak'd Soulless can create a layer surface at all. libcosmic's
/// activation-token handler also reads this var, but treats absence as
/// optional — so removing it from the env costs us focus-token polish and
/// buys us a working window.
pub static PRIVILEGED_FD: std::sync::OnceLock<Option<i32>> = std::sync::OnceLock::new();

fn main() -> cosmic::iced::Result {
    // Claim the fd BEFORE libcosmic starts and reads the env itself.
    let fd = std::env::var("X_PRIVILEGED_WAYLAND_SOCKET")
        .ok()
        .and_then(|v| v.parse::<i32>().ok());
    if fd.is_some() {
        // SAFETY: single-threaded, before any runtime starts.
        unsafe { std::env::remove_var("X_PRIVILEGED_WAYLAND_SOCKET") };
        eprintln!("[applet] claimed privileged wayland socket fd={:?}", fd);
    } else {
        eprintln!("[applet] no privileged wayland socket in env (native run, or panel did not grant)");
    }
    let _ = PRIVILEGED_FD.set(fd);

    // Flatpak warm-up: birth the blessed daemon at panel start, not first
    // click, so a keyboard shortcut pressed before any applet click finds a
    // live daemon to forward to (shortcut-first wedge, Aug 26 2026). Probe
    // with NameHasOwner -- NOT Activate -- so nothing flashes open at login.
    // SOULLESS_WARM makes a flock-losing twin exit silently instead of
    // forwarding a visible open (panel + dock applets both warm; one wins).
    // Native skips: autostart owns login there.
    if std::path::Path::new("/.flatpak-info").exists() {
        let owned = zbus::blocking::Connection::session()
            .ok()
            .and_then(|c| {
                zbus::blocking::fdo::DBusProxy::new(&c).ok().and_then(|p| {
                    p.name_has_owner("com.github.hmrdsmoke.SoullessLauncher".try_into().ok()?)
                        .ok()
                })
            })
            .unwrap_or(false);
        if !owned {
            eprintln!("[applet] warm: no daemon on the bus at panel birth -- spawning blessed");
            app::spawn_launcher_once(true);
        } else {
            eprintln!("[applet] warm: daemon already on the bus");
        }
    }

    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    // Starts the applet's event loop with `()` as the application's flags.
    cosmic::applet::run::<app::AppModel>(())
}