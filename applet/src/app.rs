// SPDX-License-Identifier: GPL-3.0-or-later
// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// applet/src/app.rs
// COSMIC panel applet - application model, update loop, and view.

use crate::config::Config;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::{window::Id, Rectangle, Subscription};
use cosmic::prelude::*;
use cosmic::widget::rectangle_tracker::{
    rectangle_tracker_subscription, RectangleTracker, RectangleUpdate,
};

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
#[derive(Default)]
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// The popup id.
    popup: Option<Id>,
    /// Configuration data that persists between application runs.
    config: Config,
    /// File organizer state
    organizer: soulless_organizer::OrganizerState,
    /// Tracks the applet button's on-screen rectangle (for launcher anchoring).
    rectangle_tracker: Option<RectangleTracker<u32>>,
    /// The applet button's current rectangle, if known.
    rectangle: Option<Rectangle>,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    OpenLauncher,
    /// No-op: returned by async Tasks (e.g. the D-Bus activate) that have no
    /// follow-up message. Handled as Task::none().
    Noop,
    PopupClosed(Id),
    Surface(cosmic::surface::Action),
    UpdateConfig(Config),
    Rectangle(RectangleUpdate<u32>),
    Organizer(soulless_organizer::Message),
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "com.github.hmrdsmoke.soulless-applet";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let app = AppModel {
            core,
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => config,
                })
                .unwrap_or_default(),
            ..Default::default()
        };

        (app, Task::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    /// Describes the interface based on the current state of the application model.
    fn view(&self) -> Element<'_, Self::Message> {
        let button = self
            .core
            .applet
            // Launcher's icon name, not the applet's: the flatpak only exports
            // icons prefixed with the app ID (com.github.hmrdsmoke.soulless-launcher),
            // so an applet-named icon can never reach the host or the sandbox
            // theme. Same art; the desktop entry already uses this name.
            .icon_button("com.github.hmrdsmoke.soulless-launcher")
            .on_press_down(Message::OpenLauncher);

        // Wrap the button in the rectangle tracker so we learn its on-screen
        // position; used to anchor the launcher under the button.
        let tracked: Element<'_, Self::Message> =
            if let Some(tracker) = self.rectangle_tracker.as_ref() {
                tracker.container(0, button).into()
            } else {
                button.into()
            };

        Element::from(self.core.applet.applet_tooltip(
            tracked,
            "soulless-launcher",
            false,
            Message::Surface,
            None,
        ))
    }

    /// Register subscriptions for this application.
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| Message::UpdateConfig(update.config)),
            soulless_organizer::subscription()
                .map(Message::Organizer),
            rectangle_tracker_subscription(0)
                .map(|update| Message::Rectangle(update.1)),
        ])
    }

    /// Handles messages emitted by the application and its widgets.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::Organizer(msg) => {
                self.organizer.update(msg);
                Task::none()
            }
            Message::UpdateConfig(config) => {
                self.config = config;
                Task::none()
            }
            Message::Rectangle(u) => {
                match u {
                    RectangleUpdate::Rectangle(r) => {
                        self.rectangle = Some(r.1);
                        eprintln!("[APPLET RECT] {:?}", self.rectangle);
                    }
                    RectangleUpdate::Init(tracker) => {
                        self.rectangle_tracker.replace(tracker);
                    }
                }
                Task::none()
            }
            Message::OpenLauncher => {
                // Button press -> activate the resident launcher daemon by calling
                // its org.freedesktop.DbusActivation.Activate method over the
                // session bus, IN-PROCESS. No child process is spawned, so there
                // is no unreaped child and no zombie (earlier spawn-based versions
                // leaked a defunct process per click because the applet never
                // wait()s on its children). Empty a{sv} platform-data arg matches
                // the method signature (busctl's `"a{sv}" 0`). Breadcrumbs prefixed
                // "[applet] activate:" so failures are greppable in the journal.
                cosmic::task::future(async move {
                    use std::collections::HashMap;
                    use zbus::zvariant::Value;
                    let platform_data: HashMap<String, Value> = HashMap::new();
                    match zbus::Connection::session().await {
                        Ok(conn) => {
                            match conn
                                .call_method(
                                    Some("com.github.hmrdsmoke.SoullessLauncher"),
                                    "/com/github/hmrdsmoke/SoullessLauncher",
                                    Some("org.freedesktop.DbusActivation"),
                                    "Activate",
                                    &platform_data,
                                )
                                .await
                            {
                                Ok(_) => {
                                    eprintln!("[applet] activate: sent to SoullessLauncher");
                                }
                                Err(e) => {
                                    eprintln!(
                                        "[applet] activate: call to SoullessLauncher failed: {e}"
                                    );
                                    // Nobody owns the name -> no daemon yet. Spawn one
                                    // ourselves, handing it the panel's privileged socket
                                    // so it inherits the CosmicPanel security-context and
                                    // can actually see zwlr_layer_shell_v1. Spawned ONCE
                                    // (guarded), and reaped so no zombie is left behind.
                                    crate::app::spawn_launcher_once(false);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("[applet] activate: session bus connection failed: {e}");
                        }
                    }
                    Message::Noop
                })
            }
            Message::Noop => Task::none(),
            Message::Surface(action) => cosmic::task::message(cosmic::Action::Cosmic(
                cosmic::app::Action::Surface(action),
            )),
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
                Task::none()
            }

        }
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}
// ── Launcher spawn (privileged-socket handoff) ───────────────────────────────

use std::sync::atomic::{AtomicBool, Ordering};

/// One spawn per applet lifetime. Two applets (panel + dock) each hold their
/// own privileged fd; whichever spawns first wins the D-Bus name, and the
/// other's Activate then succeeds normally — so no second launcher survives.
static SPAWNED: AtomicBool = AtomicBool::new(false);

/// Spawn the launcher daemon on the panel's privileged Wayland socket.
///
/// WAYLAND_SOCKET (an inherited, already-connected fd) is the standard
/// pre-connected-socket mechanism; the child connects through it and therefore
/// wears the panel's `com.system76.CosmicPanel` security-context — the identity
/// cosmic-comp exempts from protocol filtering. Without this the launcher sees
/// 35 globals instead of 56, no layer-shell, and its window never maps.
///
/// The fd must have CLOEXEC cleared or exec() closes it out from under the child.
pub fn spawn_launcher_once(warm: bool) {
    if SPAWNED.swap(true, Ordering::SeqCst) {
        eprintln!("[applet] spawn: already spawned once this session, skipping");
        return;
    }

    let Some(Some(fd)) = crate::PRIVILEGED_FD.get().copied() else {
        eprintln!("[applet] spawn: no privileged fd — cannot give the launcher layer-shell; not spawning");
        SPAWNED.store(false, Ordering::SeqCst);
        return;
    };

    // Clear CLOEXEC so the fd survives exec into the child.
    // SAFETY: fd is owned by this process and valid for its lifetime.
    unsafe {
        let flags = libc::fcntl(fd, libc::F_GETFD);
        if flags == -1 || libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) == -1 {
            eprintln!("[applet] spawn: failed to clear CLOEXEC on fd {fd}");
            SPAWNED.store(false, Ordering::SeqCst);
            return;
        }
    }

    // In-flatpak the launcher lives at /app/bin; native, it's on PATH.
    let exe = if std::path::Path::new("/app/bin/soulless-launcher").exists() {
        "/app/bin/soulless-launcher"
    } else {
        "soulless-launcher"
    };

    let mut cmd = std::process::Command::new(exe);
    cmd.env("WAYLAND_SOCKET", fd.to_string());
    // Blessing for the launcher's panel-birth guard: the applet is itself a
    // panel child, so this spawned daemon inherits COSMIC_PANEL_* — the
    // marker tells the launcher that fingerprint is legitimate here.
    cmd.env("SOULLESS_SPAWN", "applet");
    if warm {
        // Init-birth: a flock-losing twin must exit silently, not forward a
        // visible Activate -- nobody clicked anything.
        cmd.env("SOULLESS_WARM", "1");
    }
    // The child must NOT also see WAYLAND_DISPLAY, or it may prefer the
    // unprivileged socket and land right back in the filtered registry.
    cmd.env_remove("WAYLAND_DISPLAY");

    match cmd.spawn() {
        Ok(mut child) => {
            eprintln!("[applet] spawn: launched {exe} pid={} on privileged fd {fd}", child.id());
            // Reap in the background — no zombies (the defunct-process bug).
            std::thread::spawn(move || {
                let _ = child.wait();
                // Re-arm: the guard means "one live child at a time", not
                // "once per applet lifetime". Without this, a daemon that
                // dies mid-session leaves the click fallback permanently
                // fused -- dead button until re-log (stale-guard wedge).
                SPAWNED.store(false, Ordering::SeqCst);
                eprintln!("[applet] spawn: launcher exited; spawn guard re-armed");
            });
        }
        Err(e) => {
            eprintln!("[applet] spawn: failed to launch {exe}: {e}");
            SPAWNED.store(false, Ordering::SeqCst);
        }
    }
}