// GPL-3.0-or-later - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI) and Claude (Anthropic).
// Do not remove these comments.

// src/app.rs
// Application model, message types, update logic, view, and subscriptions.
// Migrated to the cosmic::Application trait (Step 1: normal window, no layer shell yet).

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
    WindowEvent(cosmic::iced::Event),
    WindowOpened(cosmic::iced::window::Id),
    Organizer(soulless_organizer::Message),
    EnterPressed,
    TabPressed,
    RequestClose,
    Noop,
}

// ── Single-instance flags (minimal, no clap) ─────────────────────────────────
// run_single_instance requires App::Flags: CosmicFlags. We don't need CLI parsing
// yet — the applet activates via D-Bus, not CLI args — so this is a minimal impl
// that satisfies the trait bounds with default args(). (CosmicFlags structure
// informed by pop-os/cosmic-launcher, GPL-3.0.)

#[derive(Debug, Clone)]
pub enum SoullessSubCommand {
    /// Reserved for a future `soulless toggle` CLI command (warm-daemon show/hide).
    #[allow(dead_code)]
    Toggle,
}

impl std::fmt::Display for SoullessSubCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SoullessSubCommand::Toggle => write!(f, "Toggle"),
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
#[derive(Clone, Debug)]
pub enum WindowKind {
    /// A right-click context menu popup. Carries the menu payload so the
    /// popup surface can render it independently of the main launcher view.
    ContextMenu(crate::search::ContextMenu),
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
    organizer:  soulless_organizer::OrganizerState,
    config:     crate::config::SoullessConfig,
    bg_handle:  Option<cosmic::iced::widget::image::Handle>,
    window_id:  cosmic::iced::window::Id,
    screen_size: Option<(u32, u32)>,
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
        (
            Self {
                core,
                search: crate::search::Search::new(),
                network:    crate::network_monitor::NetworkState::new(),
                system:     crate::system_monitor::SystemState::new(),
                hardware:   crate::hardware_monitor::HardwareMonitorState::new(),
                fps:        crate::fps_monitor::FpsMonitorState::new(),
                organizer: soulless_organizer::OrganizerState::new(),
                config,
                bg_handle,
                window_id,
                screen_size: None,
                surface_open: false,
                cursor_pos: cosmic::iced::Point::ORIGIN,
                windows: std::collections::HashMap::new(),
            },
            // Stage 2 TEST: do NOT create the surface at init. Create it on-demand
            // in dbus_activation (warm daemon) to test whether that kills the flood.
            Task::none(),
        )
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::Network(msg) => {
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
                        | crate::search::Message::RightClickDrawerBackground(..)
                        | crate::search::Message::RightClickDrawerSidebar(..)
                        | crate::search::Message::VaultOpenFileMenu(..)
                        | crate::search::Message::ShowHiddenMenu(..)
                );
                if opens_menu {
                    self.search.context_menu_pos = self.cursor_pos;
                    if let Some((w, h)) = self.screen_size {
                        self.search.window_size = (w as f32, h as f32);
                    }
                }
                if let Some(exec) = self.search.update(msg) {
                    let clean_exec = crate::utils::strip_desktop_placeholders(&exec);

                    if let Err(_e) = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&clean_exec)
                        .spawn()
                    {
                    }

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

            // ── Enable blur when window opens ─────────────────────────
            Message::WindowOpened(id) => {
                crate::ui::startup_tasks(id).map(cosmic::Action::App)
            }
            // ── Focus search bar on window focus ──────────────────────────
            Message::WindowEvent(cosmic::iced::Event::Window(
                window::Event::Focused,
            )) => {
                // DIAGNOSTIC: auto-focus disabled to test if it drives a focus->rebuild loop
                Task::none()
            }
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
                eprintln!("[LIVE] cursor=({:.0},{:.0}) screen={:?}", position.x, position.y, self.screen_size);
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
                if let Some(info) = info {
                    if let Some((w, h)) = info.logical_size {
                        self.screen_size = Some((w as u32, h as u32));
                    }
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
                        if mime_type == "text/uri-list" {
                            if matches!(
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
                        }

                        if mime_type == "text/uri-list" {
                            if let OpenDrawer::Pinned(name) = self.search.current_open_drawer.clone() {
                                let paths = crate::utils::parse_uri_list(&data);
                                if !paths.is_empty() {
                                    self.search.update(
                                        SearchMessage::FilesDroppedOnDrawer(name, paths),
                                    );
                                }
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
                if let Some(idx) = self.search.focused_app_idx {
                    if let Some(exec) = self.search.focused_exec(idx) {
                        self.search.record_launch_by_exec(&exec);
                        let clean = crate::utils::strip_desktop_placeholders(&exec);
                        let _ = std::process::Command::new("sh")
                            .arg("-c").arg(&clean).spawn();
                        return self.dismiss();
                    }
                }
                if self.search.show_search_results {
                    if let Some(exec) = self.search.focused_exec(0) {
                        self.search.record_launch_by_exec(&exec);
                        let clean = crate::utils::strip_desktop_placeholders(&exec);
                        let _ = std::process::Command::new("sh")
                            .arg("-c").arg(&clean).spawn();
                        return self.dismiss();
                    }
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
            net, sys, hw, fps,
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
        cosmic::iced::widget::container(themed)
            .width(cosmic::iced::Length::Fixed(
                crate::position::layout::WINDOW_WIDTH,
            ))
            .height(cosmic::iced::Length::Fixed(
                crate::position::layout::WINDOW_HEIGHT,
            ))
            .into()
    }

    fn dbus_activation(
        &mut self,
        msg: cosmic::dbus_activation::Message,
    ) -> Task<cosmic::Action<Self::Message>> {
        use cosmic::dbus_activation::Details;
        match msg.msg {
            Details::Activate => {
                // Warm daemon retains last session's state; reset to fresh on open.
                self.search.reset_to_default();
                self.surface_open = true;
                crate::position::placement::LauncherPosition::open(
                    self.window_id,
                    self.screen_size,
                    Message::WindowOpened,
                )
                .map(cosmic::Action::App)
            }
            _ => Task::none(),
        }
    }

    fn view_window(&self, _id: cosmic::iced::window::Id) -> Element<'_, Self::Message> {
        // NO autosize, explicit surface size. Testing if autosize itself is
        // generating internal RequestResize (size-request -> internal resize event
        // -> autosize responds -> loop, never hitting the wire).
        self.view()
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
            event::listen_with(|ev, _status, _id| match &ev {
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
            // DIAGNOSTIC: monitors disabled to test RequestResize spam source
            soulless_organizer::subscription().map(Message::Organizer),
            crate::network_monitor::subscription().map(Message::Network),
            crate::system_monitor::subscription().map(Message::System),
            crate::hardware_monitor::subscription().map(Message::Hardware),
            crate::fps_monitor::subscription().map(Message::Fps),
        ])
    }
}