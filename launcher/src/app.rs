// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI) and Claude (Anthropic).
// Do not remove these comments.
// launcher/src/app.rs
// Application model, message types, update logic, view, and subscriptions.
// Migrated to the cosmic::Application trait (Step 1: normal window, no layer shell yet).
// Window activation, surface mapping, and dismiss techniques in this file were
// adapted from System76's COSMIC applications (GPL-3.0):
//   - cosmic-launcher:   https://github.com/pop-os/cosmic-launcher
//   - cosmic-applibrary: https://github.com/pop-os/cosmic-applibrary
// Adapted: deferred surface creation (WaitingToBeShown pattern), capturing
// compositor WindowEvent::Opened/Resized for surface mapping, and the
// full-screen-stack model for click-away dismissal.

use cosmic::prelude::*;
use cosmic::iced::{
    Subscription, Task,
    event, keyboard,
    window,
};
use cosmic::iced::clipboard::dnd::{DndEvent, OfferEvent};

use crate::search::Message as SearchMessage;
use crate::network_monitor::Message as NetworkMessage;
use crate::system_monitor::Message as SystemMessage;
use crate::hardware_monitor::Message as HardwareMessage;
use crate::fps_monitor::Message as FpsMessage;

// ── Top-level message ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Message {
    Search(SearchMessage),
    Network(NetworkMessage),
    System(SystemMessage),
    Hardware(HardwareMessage),
    Fps(FpsMessage),
    Terminal(crate::terminal::Message),
    PageSelected(crate::ui::pages::Page),
    WindowEvent(cosmic::iced::Event),
    WindowOpened(cosmic::iced::window::Id),
    /// A surface's compositor configure: (surface id, logical size).
    /// Drives the region-scoped blur with per-output ground truth.
    SurfaceConfigured(cosmic::iced::window::Id, cosmic::iced::Size),
    Organizer(soulless_organizer::Message),
    EnterPressed,
    TabPressed,
    RequestClose,
    Noop,
    ShowSurface,
}

// ── Single-instance flags (minimal, no clap) ─────────────────────────────────
// run_single_instance requires App::Flags: CosmicFlags. We don't need CLI parsing
// yet — the applet activates via D-Bus, not CLI args — so this is a minimal impl
// that satisfies the trait bounds with default args(). (CosmicFlags structure
// informed by pop-os/cosmic-launcher, GPL-3.0.)

#[derive(Debug, Clone)]
pub enum SoullessSubCommand {
    /// `soulless-launcher toggle` — show the warm daemon's surface if hidden,
    /// hide it if visible. Routed to the running daemon by run_single_instance.
    Toggle,
}

impl std::fmt::Display for SoullessSubCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Lowercase: this string goes on the wire. run_single_instance sends
            // it as the D-Bus action name, and dbus_activation matches "toggle".
            SoullessSubCommand::Toggle => write!(f, "toggle"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SoullessFlags {
    pub subcommand: Option<SoullessSubCommand>,
}

impl cosmic::app::CosmicFlags for SoullessFlags {
    type SubCommand = SoullessSubCommand;
    type Args = Vec<String>;

    fn action(&self) -> Option<&Self::SubCommand> {
        self.subcommand.as_ref()
    }
}

// ── Application model ────────────────────────────────────────────────────────

/// Identifies what a secondary window (popup surface) is showing, so
/// view_window can render the correct content and update() can clean up
/// the right state when the compositor closes it.
// #30: ContextMenu/VaultMenu/VaultHiddenMenu unused while compositor-managed
// popups are shelved (reverted get_popup grab work).
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum WindowKind {
    /// A right-click context menu popup. Carries the menu payload + the drawer
    /// names needed to render it, so the popup surface is fully self-contained.
    ContextMenu(crate::search::ContextMenu, Vec<String>),
    /// A vault file-entry context menu popup (entry id + display name).
    VaultMenu(String, String),
    /// A vault hidden-app context menu popup (hidden app id).
    VaultHiddenMenu(String),
}

pub struct Soulless {
    core:       cosmic::Core,
    search:     crate::search::Search,
    network:    crate::network_monitor::NetworkState,
    system:     crate::system_monitor::SystemState,
    hardware:   crate::hardware_monitor::HardwareMonitorState,
    fps:        crate::fps_monitor::FpsMonitorState,
    terminal:   crate::terminal::TerminalBox,
    page:       crate::ui::pages::Page,
    organizer:  soulless_organizer::OrganizerState,
    config:     crate::config::SoullessConfig,
    bg_handle:  Option<cosmic::iced::widget::image::Handle>,
    window_id:  cosmic::iced::window::Id,
    dummy_id:   Option<cosmic::iced::window::Id>,
    screen_size: Option<(u32, u32)>,
    /// Compositor-reported logical size of the launcher's layer surface, from its
    /// own configure. Reliable per-output (unlike screen_size, which is whichever
    /// Output event landed last). Used to locate the launcher zone within the
    /// full-screen surface for cursor->zone coordinate conversion.
    surface_size: Option<(f32, f32)>,
    /// Whether the layer surface is currently open. Guards destroy_layer_surface
    /// so a second dismiss trigger (e.g. Unfocused right after Esc) is a no-op
    /// instead of destroying an already-destroyed surface.
    surface_open: bool,
    /// Last known cursor position (surface-relative). Updated from the existing
    /// CursorMoved subscription; used to position the context menu at the cursor.
    cursor_pos: cosmic::iced::Point,
    /// Active popup surfaces (context menus), keyed by their window id. Lets
    /// view_window render the right menu and update() clear state on close.
    windows: std::collections::HashMap<cosmic::iced::window::Id, WindowKind>,
}

impl Soulless {
    /// Guarded dismiss: only destroys the layer surface if it's currently open,
    /// so a second dismiss trigger (e.g. Unfocused right after Esc) is a safe
    /// no-op instead of destroying an already-destroyed surface.
    fn dismiss(&mut self) -> Task<cosmic::Action<Message>> {
        if self.surface_open {
            self.surface_open = false;
            crate::position::placement::LauncherPosition::close(self.window_id)
        } else {
            Task::none()
        }
    }
}

impl cosmic::Application for Soulless {
    type Executor = cosmic::executor::Default;
    type Flags = SoullessFlags;
    type Message = Message;
    // NOTE: hyphen-free APP_ID is REQUIRED for run_single_instance. cosmic derives a
    // D-Bus object path (APP_ID.replace('.', "/")) and a well-known name from this,
    // and D-Bus forbids hyphens in both. The external identity (repo, metainfo <id>,
    // .desktop, binary) keeps "soulless-launcher" — none of those read this const.
    // This only changes the Wayland app_id and the D-Bus names cosmic registers.
    const APP_ID: &'static str = "com.github.hmrdsmoke.SoullessLauncher";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        crate::config::ensure_dirs();
        crate::config::ensure_config();
        // Load the runtime theme (theme.ron over ship defaults) BEFORE any
        // view code runs. Restart the launcher to apply theme changes.
        crate::ui::theme::init(crate::config::theme_loader::load());
        let config = crate::config::load_config();
        let bg_handle = crate::config::default_background().map(|path| {
            let width = crate::position::layout::RIGHT_PANEL_WIDTH as u32;
            if let Some(rgba) = crate::config::load_background_rgba(&path, width, 900) {
                cosmic::iced::widget::image::Handle::from_rgba(width, 900, rgba)
            } else {
                cosmic::iced::widget::image::Handle::from_path(path)
            }
        });

        // Generate ONE stable surface id, store it, and create the surface with it.
        // cosmic's daemon tracks surfaces by id; an untracked (freshly-random) id is
        // why the surface floods RequestResize and won't render. Reuse this id for
        // creation and destruction. (Approach informed by pop-os/cosmic-launcher.)
        let window_id = cosmic::iced::window::Id::unique();
        let dummy_id = cosmic::iced::window::Id::unique();
        (
            Self {
                core,
                search: crate::search::Search::new(),
                network:    crate::network_monitor::NetworkState::new(),
                system:     crate::system_monitor::SystemState::new(),
                hardware:   crate::hardware_monitor::HardwareMonitorState::new(),
                fps:        crate::fps_monitor::FpsMonitorState::new(),
                terminal:   crate::terminal::TerminalBox::new(),
                page:       crate::ui::pages::Page::Monitors,
                organizer: soulless_organizer::OrganizerState::new(),
                config,
                bg_handle,
                window_id,
                dummy_id: Some(dummy_id),
                screen_size: None,
                surface_size: None,
                surface_open: false,
                cursor_pos: cosmic::iced::Point::ORIGIN,
                windows: std::collections::HashMap::new(),
            },
            // Create a DUMMY bottom-layer surface at init to anchor the launcher
            // onto the Wayland connection (esp. the inherited host socket from
            // X-HostWaylandDisplay). Mirrors cosmic-launcher. Bottom/None/empty-input
            // so it is inert and does not flood RequestResize.
            crate::position::placement::LauncherPosition::create_dummy(
                dummy_id,
                |_id| cosmic::Action::App(Message::Noop),
            ),
        )
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::Terminal(msg) => self
                .terminal
                .update(msg)
                .map(Message::Terminal)
                .map(cosmic::Action::App),

            Message::PageSelected(p) => {
                self.page = p;
                Task::none()
            }

            Message::Network(msg) => {
                // Ping runs on the blocking pool: measure() shells
                // `ping -c 4 -W 1` and parks up to ~4s. Inline in update or
                // a subscription map, that stalls the executor.
                if matches!(msg, crate::network_monitor::Message::FetchPing) {
                    return Task::perform(
                        tokio::task::spawn_blocking(crate::network_monitor::ping::measure),
                        |res| {
                            let (ping_ms, jitter_ms) = res.unwrap_or((0.0, 0.0));
                            cosmic::Action::App(Message::Network(
                                crate::network_monitor::Message::PingResult { ping_ms, jitter_ms },
                            ))
                        },
                    );
                }
                self.network.update(msg);
                Task::none()
            }

            Message::System(msg) => {
                self.system.update(msg);
                Task::none()
            }

            Message::Hardware(msg) => {
                self.hardware.update(msg);
                Task::none()
            }

            Message::Fps(msg) => {
                self.fps.update(msg);
                // Wipe decrypted plaintext for any vaulted file whose viewer has
                // closed. Cheap try_wait on tracked children; runs on the FPS tick
                // only while the vault is unlocked.
                if self.search.vault.lock_state
                    == crate::vault::VaultLockState::Unlocked
                {
                    self.search.vault.reap_finished_opens();
                }
                Task::none()
            }

            Message::Search(msg) => {
                // Capture keyboard focus-nav BEFORE msg moves into update().
                // Only keyboard nav scroll-follows; hover (FocusApp) must NOT
                // scroll or it would yank the view on mouse-over.
                let is_kbd_nav = matches!(
                    msg,
                    crate::search::Message::FocusNext | crate::search::Message::FocusPrev
                );
                // Freeze the cursor position when a right-click opens a context
                // menu, so the menu renders at the cursor and stays put. Also
                // stash window size for off-screen clamping. (cursor_pos lives
                // here in app.rs; the view only sees Search, so route it through.)
                let opens_menu = matches!(
                    msg,
                    crate::search::Message::RightClickSearchApp(..)
                        | crate::search::Message::RightClickDrawerApp(..)
                        | crate::search::Message::RightClickDrawerFile(..)
                        | crate::search::Message::RightClickDrawerSidebar(..)
                        | crate::search::Message::VaultOpenFileMenu(..)
                        | crate::search::Message::ShowHiddenMenu(..)
                );
                if opens_menu {
                    // cursor_pos is SURFACE-relative (the layer surface is full-screen).
                    // The menu overlay is stacked over the launcher content, so it fills
                    // the 700x900 launcher zone and its padding origin is THAT zone's
                    // top-left — not the screen's. Bridge the two spaces by subtracting
                    // the zone's origin, which placement::blur_rect already computes from
                    // the compositor-reported surface size. Without this the menu lands
                    // off by the zone origin — hundreds of px whenever the launcher is
                    // anchored away from the top-left corner.
                    let (ox, oy) = self
                        .surface_size
                        .map(crate::position::placement::LauncherPosition::blur_rect)
                        .map(|z| (z.x, z.y))
                        .unwrap_or((0.0, 0.0));
                    self.search.context_menu_pos = cosmic::iced::Point::new(
                        self.cursor_pos.x - ox,
                        self.cursor_pos.y - oy,
                    );
                }
                if let Some(exec) = self.search.update(msg) {
                    crate::utils::spawn_exec(&exec);

                    self.dismiss()
                } else if is_kbd_nav {
                    // One discrete snap per keypress keeps the focused item in
                    // view without continuous re-render (no flood).
                    if let Some(idx) = self.search.focused_app_idx {
                        let len = self.search.current_grid_len();
                        eprintln!("[SCROLL] kbd_nav idx={} len={}", idx, len);
                        if len > 1 {
                            let y = idx as f32 / (len - 1) as f32;
                            return cosmic::iced::widget::scrollable::snap_to(
                                cosmic::widget::Id::new("soulless-results-scroll"),
                                cosmic::iced::widget::scrollable::RelativeOffset { x: Some(0.0), y: Some(y) },
                            )
                            .map(cosmic::Action::App);
                        }
                    }
                    Task::none()
                } else {
                    Task::none()
                }
            }

            // ── Focus when the REAL launcher window opens ──────────────
            // Blur is NOT set here: it rides the surface's own configure
            // (SurfaceConfigured below), so the rect always uses the
            // compositor-reported size for the output it actually landed on.
            Message::WindowOpened(id) if id == self.window_id => {
                crate::ui::startup_tasks(id).map(cosmic::Action::App)
            }
            Message::WindowOpened(_) => Task::none(),
            Message::ShowSurface => {
                eprintln!("[launcher] ShowSurface: creating real surface now (deferred)");
                crate::position::placement::LauncherPosition::open(
                    self.window_id,
                    self.screen_size,
                    Message::WindowOpened,
                )
                .map(cosmic::Action::App)
            }
            // ── Focus search bar on window focus ──────────────────────────
            Message::WindowEvent(cosmic::iced::Event::Window(
                window::Event::Focused,
            )) => {
                // DIAGNOSTIC: auto-focus disabled to test if it drives a focus->rebuild loop
                Task::none()
            }
            // ── Surface mapping (Opened/Resized) ──────────────────────────
            // The compositor Opened/Resized events complete the layer-surface
            // handshake so it MAPS (becomes visible). cosmic-launcher handles
            // these. Quiet return (Task::none) so no RequestResize flood.
            // ── Surface configure → region blur (per-output truth) ────────
            // Opened/Resized carry the compositor-assigned LOGICAL size of
            // the surface that was configured. Real window: size the blur
            // region from it. Dummy or anything else: quiet no-op. Resized
            // re-fires so blur tracks size changes; backend updates in place.
            Message::SurfaceConfigured(id, size) if id == self.window_id => {
                eprintln!(
                    "[launcher] SurfaceConfigured {}x{} -> blur rect",
                    size.width, size.height
                );
                self.surface_size = Some((size.width, size.height));
                let zone = crate::position::placement::LauncherPosition::blur_rect((
                    size.width,
                    size.height,
                ));
                crate::ui::blur_task(id, zone).map(cosmic::Action::App)
            }
            Message::SurfaceConfigured(_, _) => Task::none(),
            // ── Keyboard ──────────────────────────────────────────────────
            Message::WindowEvent(
                cosmic::iced::Event::Keyboard(
                    keyboard::Event::KeyPressed { key, modifiers, .. },
                ),
            ) => {
                crate::keybinds::actions::handle_key(
                    &key,
                    modifiers,
                    &mut self.search,
                    Message::Search,
                    || cosmic::task::message(Message::RequestClose),
                ).map(cosmic::Action::App)
            }

            // ── Track cursor position ────────────────────────────────────
            Message::WindowEvent(cosmic::iced::Event::Mouse(
                cosmic::iced::mouse::Event::CursorMoved { position },
            )) => {
                // STORM RE-TEST (layer-shell now stable): store cursor_pos on motion
                // and see if the pointer storm returns. Previously this looped:
                // state change per motion -> re-render -> surface perturb -> re-emit.
                self.cursor_pos = position;
                Task::none()
            }

            // ── Layer surface lost focus → close (layer-shell dismiss) ───
            // CRITICAL: a layer surface does NOT emit window::Event::Unfocused.
            // It emits its own LayerEvent::Unfocused through the Wayland
            // platform-specific event channel. This is how cosmic-launcher
            // itself detects click-away dismiss. Matching window::Event here
            // never fires for a layer surface — must match the wayland path.
            // Capture monitor geometry from Output events so the surface can be sized
            // and anchored relative to the real screen instead of blind hardcoded
            // constants. Both Created and InfoUpdate carry OutputInfo.logical_size.
            Message::WindowEvent(cosmic::iced::Event::PlatformSpecific(
                cosmic::iced::event::PlatformSpecific::Wayland(
                    cosmic::iced::event::wayland::Event::Output(output_event, _wl_output),
                ),
            )) => {
                use cosmic::iced::event::wayland::OutputEvent;
                let info = match output_event {
                    OutputEvent::Created(Some(i)) => Some(i),
                    OutputEvent::InfoUpdate(i) => Some(i),
                    _ => None,
                };
                if let Some(info) = info
                    && let Some((w, h)) = info.logical_size {
                        self.screen_size = Some((w as u32, h as u32));
                    }
                Task::none()
            }
            Message::WindowEvent(cosmic::iced::Event::PlatformSpecific(
                cosmic::iced::event::PlatformSpecific::Wayland(
                    cosmic::iced::event::wayland::Event::Layer(
                        cosmic::iced::event::wayland::LayerEvent::Unfocused,
                        _surface,
                        _id,
                    ),
                ),
            )) => {
                self.dismiss()
            }

            // ── Drag-and-drop ────────────────────────────────────────────
            Message::WindowEvent(cosmic::iced::Event::Dnd(dnd_event)) => {
                use crate::search::OpenDrawer;

                match dnd_event {
                    DndEvent::Offer(_, OfferEvent::Enter { .. }) => {
                        match &self.search.current_open_drawer {
                            OpenDrawer::Vault => {
                                self.search.update(
                                    SearchMessage::VaultDragHover(true),
                                );
                            }
                            OpenDrawer::Pinned(name) => {
                                let name = name.clone();
                                self.search.update(
                                    SearchMessage::DrawerDragHover(Some(name.clone())),
                                );
                                self.search.update(
                                    SearchMessage::DrawerFileHover(Some(name)),
                                );
                            }
                            OpenDrawer::Search => {}
                        }
                    }

                    DndEvent::Offer(_, OfferEvent::Leave)
                    | DndEvent::Offer(_, OfferEvent::LeaveDestination) => {
                        self.search
                            .update(SearchMessage::VaultDragHover(false));
                        self.search
                            .update(SearchMessage::DrawerDragHover(None));
                        self.search
                            .update(SearchMessage::DrawerFileHover(None));
                    }

                    DndEvent::Offer(_, OfferEvent::Data {
                        data,
                        mime_type,
                    }) => {
                        if mime_type == "text/uri-list"
                            && matches!(
                                self.search.current_open_drawer,
                                OpenDrawer::Vault
                            ) {
                                let paths = crate::utils::parse_uri_list(&data);

                                if !paths.is_empty() {
                                    self.search.update(
                                        SearchMessage::VaultFilesDropped(paths),
                                    );
                                }
                            }

                        if mime_type == "text/uri-list"
                            && let OpenDrawer::Pinned(name) = self.search.current_open_drawer.clone() {
                                let paths = crate::utils::parse_uri_list(&data);
                                if !paths.is_empty() {
                                    self.search.update(
                                        SearchMessage::FilesDroppedOnDrawer(name, paths),
                                    );
                                }
                            }
                        self.search
                            .update(SearchMessage::VaultDragHover(false));
                        self.search
                            .update(SearchMessage::DrawerDragHover(None));
                        self.search
                            .update(SearchMessage::DrawerFileHover(None));
                    }

                    _ => {}
                }

                Task::none()
            }

            Message::RequestClose => {
                self.dismiss()
            }
            Message::Noop => Task::none(),
            Message::TabPressed => {
                // Tab cycles: Search -> drawer0 -> ... -> lastDrawer -> Search ...
                // Handled on KEY RELEASE so autorepeat can't skip stops per press.
                // The search bar is a real stop in the loop (after the last drawer).
                let drawers = self.search.drawer_state.drawers();
                if drawers.is_empty() {
                    // No drawers: Tab just focuses the search bar.
                    self.search
                        .update(crate::search::Message::DrawerClicked(String::new()));
                    return cosmic::widget::text_input::focus(cosmic::widget::Id::new(
                        "soulless-search-bar",
                    ))
                    .map(cosmic::Action::App);
                }

                // Where are we now? Some(i) = on drawer i; None = on Search/other.
                let current_idx = if let crate::search::OpenDrawer::Pinned(name) =
                    &self.search.current_open_drawer
                {
                    drawers.iter().position(|d| &d.name == name)
                } else {
                    None
                };

                match current_idx {
                    // On a drawer that's NOT the last -> next drawer.
                    Some(i) if i + 1 < drawers.len() => {
                        let next_name = drawers[i + 1].name.clone();
                        self.search.update(crate::search::Message::DrawerClicked(
                            next_name.clone(),
                        ));
                        cosmic::widget::button::focus(cosmic::widget::Id::new(format!(
                            "drawer-btn-{}",
                            next_name
                        )))
                        .map(cosmic::Action::App)
                    }
                    // On the LAST drawer -> wrap to the search bar.
                    Some(_) => {
                        self.search.current_open_drawer = crate::search::OpenDrawer::Search;
                        cosmic::widget::text_input::focus(cosmic::widget::Id::new(
                            "soulless-search-bar",
                        ))
                        .map(cosmic::Action::App)
                    }
                    // On Search (or anything else) -> first drawer.
                    None => {
                        let next_name = drawers[0].name.clone();
                        self.search.update(crate::search::Message::DrawerClicked(
                            next_name.clone(),
                        ));
                        cosmic::widget::button::focus(cosmic::widget::Id::new(format!(
                            "drawer-btn-{}",
                            next_name
                        )))
                        .map(cosmic::Action::App)
                    }
                }
            }
            Message::EnterPressed => {
                // Terminal page owns Enter: the box's on_submit runs the
                // command. Without this gate the global listener would ALSO
                // launch the focused search result and dismiss the window.
                if self.page == crate::ui::pages::Page::Terminal {
                    return Task::none();
                }
                if let Some(idx) = self.search.focused_app_idx
                    && let Some(exec) = self.search.focused_exec(idx) {
                        self.search.record_launch_by_exec(&exec);
                        crate::utils::spawn_exec(&exec);
                        return self.dismiss();
                    }
                // Type-and-Enter: launch the top result ONLY when the user has
                // actually typed a query. Without the non-empty guard, a bare
                // Enter on a freshly-opened launcher (query empty, all apps
                // showing, nothing focused) launches result 0 — that is how a
                // stray Enter (e.g. the Enter that ran a terminal `busctl`
                // activation) leaked into the window and launched the first app.
                // Gating on a non-empty query preserves type-and-Enter while
                // making a bare Enter on the fresh window a no-op.
                if !self.search.query.trim().is_empty()
                    && self.search.show_search_results
                    && let Some(exec) = self.search.focused_exec(0) {
                        self.search.record_launch_by_exec(&exec);
                        crate::utils::spawn_exec(&exec);
                        return self.dismiss();
                    }
                Task::none()
            }
            Message::Organizer(msg) => {
                self.organizer.update(msg);
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let (toolbox, right, _drop_zone, menu_overlay) = crate::drawers::view(&self.search);
        let page_area: cosmic::iced::Element<'_, Message> = match self.page {
            crate::ui::pages::Page::Monitors => {
                let net = if self.config.show_system_monitor {
                    crate::network_monitor::view(&self.network).map(Message::Network)
                } else { cosmic::iced::widget::space::horizontal().into() };
                let sys = if self.config.show_system_monitor {
                    crate::system_monitor::view(&self.system).map(Message::System)
                } else { cosmic::iced::widget::space::horizontal().into() };
                let hw = if self.config.show_system_monitor {
                    crate::hardware_monitor::view(&self.hardware).map(Message::Hardware)
                } else { cosmic::iced::widget::space::horizontal().into() };
                let fps = if self.config.show_system_monitor {
                    crate::fps_monitor::view(&self.fps).map(Message::Fps)
                } else { cosmic::iced::widget::space::horizontal().into() };
                crate::ui::panels::monitor_grid(net, sys, hw, fps)
            }
            crate::ui::pages::Page::Terminal => crate::ui::panels::terminal_frame(
                crate::terminal::view(&self.terminal).map(Message::Terminal),
            ),
        };
        let dots = crate::ui::pages::dot_strip(self.page, Message::PageSelected);
        let banner = if self.config.organizer_enabled {
            crate::ui::organizer::organizer_banner(&self.organizer, Message::Organizer)
        } else { None };
        let composed = crate::ui::panels::compose(
            {
                let t = toolbox.map(Message::Search);
                use cosmic::iced::widget::Column;
                let mut col = Column::new().push(t);
                if let Some(b) = banner {
                    col = col.push(b);
                }
                col.into()
            },
            right.map(Message::Search),
            dots,
            page_area,
            self.bg_handle.clone(),
        );
        let composed = match menu_overlay {
            Some(m) => cosmic::iced::widget::stack([
                composed,
                m.map(Message::Search),
            ])
            .into(),
            None => composed,
        };
        // Bridge: compose() yields a cosmic::iced::Theme element; the
        // cosmic::Application trait's view() expects a cosmic::Theme element.
        // Themer wraps the inner (iced-themed) tree so it can live in the
        // cosmic-themed outer tree.
        let themed = cosmic::iced::widget::Themer::new(
            None::<cosmic::iced::Theme>,
            composed,
        );
        // Constrain the view root to a HARD fixed size (no Fill at the outermost
        // level). This gives autosize a STABLE measurement that doesn't change
        // frame-to-frame, so it acks the compositor once and settles instead of
        // re-requesting every frame (the flood). Fill content lives inside.
        // The launcher content, fixed-size (the NON-DISMISS zone). Wrapped in a
        // mouse_area that absorbs clicks so they do NOT fall through to the
        // background dismiss layer below.
        let launcher_zone = cosmic::iced::widget::mouse_area(
            cosmic::iced::widget::container(themed)
                .width(cosmic::iced::Length::Fixed(
                    crate::position::layout::WINDOW_WIDTH,
                ))
                .height(cosmic::iced::Length::Fixed(
                    crate::position::layout::WINDOW_HEIGHT,
                )),
        )
        .on_press(Message::Noop);

        // Read the applet position from LIVE config (dynamic, no hardcoding) and
        // derive the launcher placement: wing -> horizontal, bar edge -> vertical.
        let (edge, wing, bar_px) = crate::position::placement::LauncherPosition::applet_position();
        // Orientation-aware placement. Horizontal bar (Top/Bottom): wings run
        // left/right, edge is top/bottom. Vertical bar (Left/Right): wings run
        // top/bottom, edge is left/right. Map each axis accordingly.
        use cosmic::iced::platform_specific::shell::commands::layer_surface::Anchor as LSAnchor;
        use crate::position::placement::Wing;
        let horizontal_bar = matches!(edge, LSAnchor::TOP | LSAnchor::BOTTOM);
        let (align_x, align_y) = if horizontal_bar {
            let ax = match wing {
                Wing::First => cosmic::iced::Alignment::Start,
                Wing::Second => cosmic::iced::Alignment::End,
                Wing::Center => cosmic::iced::Alignment::Center,
            };
            let ay = if matches!(edge, LSAnchor::TOP) {
                cosmic::iced::Alignment::Start
            } else {
                cosmic::iced::Alignment::End
            };
            (ax, ay)
        } else {
            let ay = match wing {
                Wing::First => cosmic::iced::Alignment::Start,
                Wing::Second => cosmic::iced::Alignment::End,
                Wing::Center => cosmic::iced::Alignment::Center,
            };
            let ax = if matches!(edge, LSAnchor::LEFT) {
                cosmic::iced::Alignment::Start
            } else {
                cosmic::iced::Alignment::End
            };
            (ax, ay)
        };
        // Clear the bar on its edge using the bar's ACTUAL thickness (from config),
        // small gap on the other edges. Launcher sits just past the panel/dock
        // wherever it is and whatever size it is set to.
        // Flush placement: zero gap puts the launcher touching the bar and the
        // screen corner. MUST match placement::blur_rect's gap — blur region and
        // cursor->zone conversion both derive from the same constant.
        let gap = crate::ui::theme::get().window_gap;
        let bar_pad = bar_px as f32 + gap;
        let padding = match edge {
            cosmic::iced::platform_specific::shell::commands::layer_surface::Anchor::TOP =>
                cosmic::iced::Padding { top: bar_pad, right: gap, bottom: gap, left: gap },
            cosmic::iced::platform_specific::shell::commands::layer_surface::Anchor::BOTTOM =>
                cosmic::iced::Padding { top: gap, right: gap, bottom: bar_pad, left: gap },
            cosmic::iced::platform_specific::shell::commands::layer_surface::Anchor::LEFT =>
                cosmic::iced::Padding { top: gap, right: gap, bottom: gap, left: bar_pad },
            cosmic::iced::platform_specific::shell::commands::layer_surface::Anchor::RIGHT =>
                cosmic::iced::Padding { top: gap, right: bar_pad, bottom: gap, left: gap },
            _ => cosmic::iced::Padding::from(gap),
        };

        // Full-screen stack: background dismisses (click outside launcher zone),
        // launcher zone does not. Positioning via the vertical space + align_x.
        // (Matches cosmic-applibrary root layout.)
        cosmic::iced::widget::stack![
            // Background: click anywhere outside the launcher -> dismiss.
            cosmic::iced::widget::mouse_area(
                cosmic::iced::widget::container(cosmic::iced::widget::space::horizontal())
                    .width(cosmic::iced::Length::Fill)
                    .height(cosmic::iced::Length::Fill)
            )
            .on_press(Message::RequestClose),
            // Positioned launcher zone: aligned to the applet corner (dynamic).
            cosmic::iced::widget::container(launcher_zone)
                .align_x(align_x)
                .align_y(align_y)
                .padding(padding)
                .width(cosmic::iced::Length::Fill)
                .height(cosmic::iced::Length::Fill)
        ]
        .width(cosmic::iced::Length::Fill)
        .height(cosmic::iced::Length::Fill)
        .into()
    }

    fn dbus_activation(
        &mut self,
        msg: cosmic::dbus_activation::Message,
    ) -> Task<cosmic::Action<Self::Message>> {
        use cosmic::dbus_activation::Details;
        match msg.msg {
            // Show: idempotent. A second Activate while the surface is already
            // open is a no-op (NOT a re-open). This is the fix for the double-
            // activation bug: an unguarded re-open reset search + re-ran open()
            // on an already-live window, which surfaced as "launches the first
            // result and vanishes." If already open, do nothing.
            Details::Activate => {
                eprintln!("[launcher] dbus_activation: Activate received, surface_open={}", self.surface_open);
                if self.surface_open {
                    Task::none()
                } else {
                    // Defer surface creation to the next event-loop cycle (like
                    // cosmic-launcher, which defers via its search-response round-trip).
                    // Creating the layer surface synchronously here does NOT map it.
                    self.search.refresh_index();
                    self.search.reset_to_default();
                    self.terminal.reset();
                    self.page = crate::ui::pages::Page::Monitors;
                    self.surface_open = true;
                    eprintln!("[launcher] deferring surface creation via ShowSurface");
                    cosmic::task::message(cosmic::Action::App(Message::ShowSurface))
                }
            }
            // Toggle: flip show/hide. Open -> dismiss; closed -> show fresh.
            // Reached via `ActivateAction("toggle")` (e.g. a future CLI
            // `soulless toggle` or a panel-button toggle press). The action
            // string is matched explicitly so unknown actions are ignored.
            Details::ActivateAction { action, .. } if action == "toggle" => {
                if self.surface_open {
                    self.dismiss()
                } else {
                    self.search.refresh_index();
                    self.search.reset_to_default();
                    self.terminal.reset();
                    self.page = crate::ui::pages::Page::Monitors;
                    self.surface_open = true;
                    crate::position::placement::LauncherPosition::open(
                        self.window_id,
                        self.screen_size,
                        Message::WindowOpened,
                    )
                    .map(cosmic::Action::App)
                }
            }
            _ => Task::none(),
        }
    }

    fn view_window(&self, id: cosmic::iced::window::Id) -> Element<'_, Self::Message> {
        // If this window id is a registered popup surface, render that menu's
        // content. Otherwise it's the main launcher surface -> normal view.
        // Each menu popup is built with iced-themed widgets; bridge through
        // Themer so the returned element is cosmic-themed like the main view.
        // (Inlined per-arm rather than via a closure so the input/output
        // lifetimes tie together for the borrow checker.)
        match self.windows.get(&id) {
            Some(WindowKind::ContextMenu(menu, drawer_names)) => {
                cosmic::iced::widget::Themer::new(
                    None::<cosmic::iced::Theme>,
                    crate::drawers::context_menu_popup(menu, drawer_names)
                        .map(Message::Search),
                )
                .into()
            }
            Some(WindowKind::VaultMenu(entry_id, name)) => {
                cosmic::iced::widget::Themer::new(
                    None::<cosmic::iced::Theme>,
                    crate::vault::ui::vault_menu_popup(entry_id, name)
                        .map(Message::Search),
                )
                .into()
            }
            Some(WindowKind::VaultHiddenMenu(app_id)) => {
                cosmic::iced::widget::Themer::new(
                    None::<cosmic::iced::Theme>,
                    crate::vault::ui::vault_hidden_menu_popup(app_id)
                        .map(Message::Search),
                )
                .into()
            }
            None if Some(id) == self.dummy_id => {
                // The dummy anchor surface: render NOTHING so it stays invisible
                // (like cosmic-launcher). It exists only to anchor the Wayland
                // connection (inherited host socket); drawing the launcher into
                // it caused the partial, non-interactive ghost window.
                cosmic::iced::widget::column![].into()
            }
            None => self.view(),
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch([
            crate::keep_alive::subscription(),
            // CRITICAL: filter events. A blanket listen().map(WindowEvent) forwards
            // EVERY event — including per-frame redraw events — back into the message
            // loop, and each message re-arms a render => self-sustaining ~110fps
            // invalidation flood. Return Some ONLY for events we actually handle;
            // everything else (frame events especially) returns None and the loop
            // settles. (Matches cosmic-launcher's listen_raw filtering.)
            event::listen_with(|ev, _status, id| match &ev {
                // Layer surface events (dismiss on Unfocused)
                cosmic::iced::Event::PlatformSpecific(
                    cosmic::iced::event::PlatformSpecific::Wayland(
                        cosmic::iced::event::wayland::Event::Layer(..),
                    ),
                ) => Some(Message::WindowEvent(ev)),
                // Output events (monitor geometry capture)
                cosmic::iced::Event::PlatformSpecific(
                    cosmic::iced::event::PlatformSpecific::Wayland(
                        cosmic::iced::event::wayland::Event::Output(..),
                    ),
                ) => Some(Message::WindowEvent(ev)),
                // Mouse motion (cursor tracking)
                cosmic::iced::Event::Mouse(cosmic::iced::mouse::Event::CursorMoved { .. }) => {
                    Some(Message::WindowEvent(ev))
                }
                cosmic::iced::Event::Keyboard(
                    cosmic::iced::keyboard::Event::KeyPressed { .. },
                ) => Some(Message::WindowEvent(ev)),
                // Surface mapping: the compositor.s Opened/Resized events complete
                // the layer-surface handshake. WITHOUT capturing these, the surface
                // is CREATED but never MAPPED = invisible window. cosmic-launcher
                // captures both. Handled quietly in update() so no flood.
                cosmic::iced::Event::Window(cosmic::iced::window::Event::Opened { size, .. }) => {
                    Some(Message::SurfaceConfigured(id, *size))
                }
                cosmic::iced::Event::Window(cosmic::iced::window::Event::Resized(size)) => {
                    Some(Message::SurfaceConfigured(id, *size))
                }
                // Everything else (frame/redraw/etc.) -> None. Breaks the flood.
                _ => None,
            }),
            cosmic::iced::keyboard::listen().map(|event| {
                match event {
                    cosmic::iced::keyboard::Event::KeyReleased {
                        key: cosmic::iced::keyboard::Key::Named(cosmic::iced::keyboard::key::Named::Enter),
                        ..
                    } => Message::EnterPressed,
                    cosmic::iced::keyboard::Event::KeyReleased {
                        key: cosmic::iced::keyboard::Key::Named(cosmic::iced::keyboard::key::Named::Tab),
                        modifiers,
                        ..
                    } if !modifiers.shift() => Message::TabPressed,
                    _ => Message::Noop,
                }
            }),

            cosmic::iced::window::open_events().map(Message::WindowOpened),
            // Organizer stays always-on — watching Downloads while hidden is
            // its entire job.
            soulless_organizer::subscription().map(Message::Organizer),
            // Monitors: alive only while the surface is up. The resident
            // Monitor census: ungated on purpose. Pure event-driven — no
            // timers, no sampling — and output Created fires once at
            // registry bind, so a gated subscriber starts deaf. Proven by
            // the applet-spawned instance catching both outputs at spawn.
            crate::fps_monitor::monitors_subscription().map(Message::Fps),
            // daemon spends most of its life hidden — no sampling, no
            // nvidia-smi/df/ping spawns, no 16ms FPS heartbeat while nothing
            // is on screen. Subscription diffing starts/stops these cleanly
            // on open/close; histories freeze and resume.
            if self.surface_open {
                Subscription::batch([
                    crate::network_monitor::subscription().map(Message::Network),
                    crate::system_monitor::subscription().map(Message::System),
                    crate::hardware_monitor::subscription().map(Message::Hardware),
                    crate::fps_monitor::subscription().map(Message::Fps),
                ])
            } else {
                Subscription::none()
            },
        ])
    }
}