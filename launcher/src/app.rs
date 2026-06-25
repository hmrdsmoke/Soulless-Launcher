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

pub struct Soulless {
    core:       cosmic::Core,
    search:     crate::search::Search,
    network:    crate::network_monitor::NetworkState,
    system:     crate::system_monitor::SystemState,
    hardware:   crate::hardware_monitor::HardwareMonitorState,
    fps:        crate::fps_monitor::FpsMonitorState,
    organizer:  soulless_organizer::OrganizerState,
    config:     crate::config::SoullessConfig,
    cursor_pos: Option<cosmic::iced::Point>,
    bg_handle:  Option<cosmic::iced::widget::image::Handle>,
    window_id:  cosmic::iced::window::Id,
}

impl cosmic::Application for Soulless {
    type Executor = cosmic::executor::Default;
    type Flags = SoullessFlags;
    type Message = Message;
    const APP_ID: &'static str = "com.github.hmrdsmoke.soulless-launcher";

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
                cursor_pos: None,
                bg_handle,
                window_id,
            },
            crate::position::placement::LauncherPosition::open(window_id, Message::WindowOpened)
                .map(cosmic::Action::App),
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
                if let Some(exec) = self.search.update(msg) {
                    let clean_exec = crate::utils::strip_desktop_placeholders(&exec);

                    if let Err(_e) = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&clean_exec)
                        .spawn()
                    {
                    }

                    cosmic::task::message(cosmic::Action::Cosmic(
                        cosmic::app::Action::Close,
                    ))
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
                eprintln!("[FOCUSED] received (auto-focus disabled for test)");
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
                self.cursor_pos = Some(position);
                Task::none()
            }

            // ── Layer surface lost focus → close (layer-shell dismiss) ───
            // CRITICAL: a layer surface does NOT emit window::Event::Unfocused.
            // It emits its own LayerEvent::Unfocused through the Wayland
            // platform-specific event channel. This is how cosmic-launcher
            // itself detects click-away dismiss. Matching window::Event here
            // never fires for a layer surface — must match the wayland path.
            Message::WindowEvent(cosmic::iced::Event::PlatformSpecific(
                cosmic::iced::event::PlatformSpecific::Wayland(
                    cosmic::iced::event::wayland::Event::Layer(
                        cosmic::iced::event::wayland::LayerEvent::Unfocused,
                        _surface,
                        _id,
                    ),
                ),
            )) => {
                eprintln!("[HANDLER] LayerEvent::Unfocused HIT -> issuing Close");
                cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Close,
                ))
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
                cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Close,
                ))
            }
            Message::Noop => Task::none(),
            Message::EnterPressed => {
                if let Some(idx) = self.search.focused_app_idx {
                    if let Some(exec) = self.search.focused_exec(idx) {
                        self.search.record_launch_by_exec(&exec);
                        let clean = crate::utils::strip_desktop_placeholders(&exec);
                        let _ = std::process::Command::new("sh")
                            .arg("-c").arg(&clean).spawn();
                        return cosmic::task::message(cosmic::Action::Cosmic(
                            cosmic::app::Action::Close,
                        ));
                    }
                }
                if self.search.show_search_results {
                    if let Some(exec) = self.search.focused_exec(0) {
                        self.search.record_launch_by_exec(&exec);
                        let clean = crate::utils::strip_desktop_placeholders(&exec);
                        let _ = std::process::Command::new("sh")
                            .arg("-c").arg(&clean).spawn();
                        return cosmic::task::message(cosmic::Action::Cosmic(
                            cosmic::app::Action::Close,
                        ));
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
        let (toolbox, right, _drop_zone) = crate::drawers::view(&self.search);
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
        // Bridge: compose() yields a cosmic::iced::Theme element; the
        // cosmic::Application trait's view() expects a cosmic::Theme element.
        // Themer wraps the inner (iced-themed) tree so it can live in the
        // cosmic-themed outer tree.
        cosmic::iced::widget::Themer::new(
            None::<cosmic::iced::Theme>,
            composed,
        )
        .into()
    }

    fn dbus_activation(
        &mut self,
        _msg: cosmic::dbus_activation::Message,
    ) -> Task<cosmic::Action<Self::Message>> {
        eprintln!("[DBUS] activation received");
        Task::none()
    }

    fn view_window(&self, _id: cosmic::iced::window::Id) -> Element<'_, Self::Message> {
        // Under no_main_window (layer shell), cosmic renders our surface through
        // view_window(), NOT view(). The trait default panics, so we MUST override
        // it. Delegate to the same content view() produces.
        self.view()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch([
            crate::keep_alive::subscription(),
            event::listen().map(|ev| {
                // DIAGNOSTIC: log window + wayland events to see what fires on click-away.
                match &ev {
                    cosmic::iced::Event::Window(we) => {
                        eprintln!("[EVT] Window: {:?}", we);
                    }
                    cosmic::iced::Event::PlatformSpecific(
                        cosmic::iced::event::PlatformSpecific::Wayland(we),
                    ) => {
                        eprintln!("[EVT] Wayland: {:?}", we);
                    }
                    _ => {}
                }
                Message::WindowEvent(ev)
            }),
            cosmic::iced::keyboard::listen().map(|event| {
                match event {
                    cosmic::iced::keyboard::Event::KeyReleased {
                        key: cosmic::iced::keyboard::Key::Named(cosmic::iced::keyboard::key::Named::Enter),
                        ..
                    } => Message::EnterPressed,
                    _ => Message::Noop,
                }
            }),

            cosmic::iced::window::open_events().map(Message::WindowOpened),
            // DIAGNOSTIC: monitors disabled to test RequestResize spam source
            // soulless_organizer::subscription().map(Message::Organizer),
            // crate::network_monitor::subscription().map(Message::Network),
            // crate::system_monitor::subscription().map(Message::System),
            // crate::hardware_monitor::subscription().map(Message::Hardware),
            // crate::fps_monitor::subscription().map(Message::Fps),
        ])
    }
}