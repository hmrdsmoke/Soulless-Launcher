// GPL-3.0-or-later - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

// src/app.rs
// Application model, message types, update logic, view, and subscriptions.

use cosmic::iced::{
    Element, Subscription, Task, Theme,
    event, keyboard,
    window,
};
use cosmic::iced::clipboard::dnd::{DndEvent, OfferEvent};

use crate::position::LauncherPosition;
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
    Noop,
}

// ── Application model ────────────────────────────────────────────────────────

pub struct Soulless {
    search:     crate::search::Search,
    network:    crate::network_monitor::NetworkState,
    system:     crate::system_monitor::SystemState,
    hardware:   crate::hardware_monitor::HardwareMonitorState,
    fps:        crate::fps_monitor::FpsMonitorState,
    organizer: soulless_organizer::OrganizerState,
    config: crate::config::SoullessConfig,
    cursor_pos: Option<cosmic::iced::Point>,
    bg_handle:  Option<cosmic::iced::widget::image::Handle>,
}

impl Soulless {
    pub fn new() -> (Self, Task<Message>) {
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

        (
            Self {
                search: crate::search::Search::new(),
                network:    crate::network_monitor::NetworkState::new(),
                system:     crate::system_monitor::SystemState::new(),
                hardware:   crate::hardware_monitor::HardwareMonitorState::new(),
                fps:        crate::fps_monitor::FpsMonitorState::new(),
                organizer: soulless_organizer::OrganizerState::new(),
                config,
                cursor_pos: None,
                bg_handle,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
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

                    cosmic::iced::exit::<Message>()
                } else {
                    Task::none()
                }
            }

            // ── Enable blur when window opens ─────────────────────────
            Message::WindowOpened(id) => {
                return crate::ui::startup_tasks(id);
            }
            // ── Focus search bar on window focus ──────────────────────────
            Message::WindowEvent(cosmic::iced::Event::Window(
                window::Event::Focused,
            )) => {
                return cosmic::widget::text_input::focus(
                    cosmic::widget::Id::new("soulless-search-bar")
                );
            }
            // ── Keyboard ──────────────────────────────────────────────────
            Message::WindowEvent(
                cosmic::iced::Event::Keyboard(
                    keyboard::Event::KeyPressed { key, modifiers, .. },
                ),
            ) => {
                return crate::keybinds::actions::handle_key(
                    &key,
                    modifiers,
                    &mut self.search,
                    Message::Search,
                    || cosmic::iced::exit::<Message>(),
                );
            }

            // ── Track cursor position ────────────────────────────────────
            Message::WindowEvent(cosmic::iced::Event::Mouse(
                cosmic::iced::mouse::Event::CursorMoved { position },
            )) => {
                self.cursor_pos = Some(position);
                Task::none()
            }

            // ── Click outside → exit ─────────────────────────────────────
            Message::WindowEvent(cosmic::iced::Event::Mouse(
                cosmic::iced::mouse::Event::ButtonPressed(_),
            )) => {
                let size = LauncherPosition.window_size();

                let outside = self.cursor_pos.map_or(false, |p| {
                    p.x < 0.0
                        || p.y < 0.0
                        || p.x > size.width
                        || p.y > size.height
                });

                if outside {
                    return cosmic::iced::exit::<Message>();
                }

                Task::none()
            }

            // ── Drag-and-drop ────────────────────────────────────────────
            // NOTE: drawer file drops are handled entirely by drawers.rs via
            // dnd_destination on_finish. main.rs only handles vault drops and
            // hover state updates here.
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
                        // Only handle vault drops here — drawer drops are
                        // handled by dnd_destination on_finish in drawers.rs.
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

                        // Handle drawer file drops
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
                        // Clear all hover states after drop
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

            Message::Noop => Task::none(),
            Message::EnterPressed => {
                if let Some(idx) = self.search.focused_app_idx {
                    if let Some(exec) = self.search.focused_exec(idx) {
                        self.search.record_launch_by_exec(&exec);
                        let clean = crate::utils::strip_desktop_placeholders(&exec);
                        let _ = std::process::Command::new("sh")
                            .arg("-c").arg(&clean).spawn();
                        return cosmic::iced::exit();
                    }
                }
                // No focused app — launch top search result if searching
                if self.search.show_search_results {
                    if let Some(exec) = self.search.focused_exec(0) {
                        self.search.record_launch_by_exec(&exec);
                        let clean = crate::utils::strip_desktop_placeholders(&exec);
                        let _ = std::process::Command::new("sh")
                            .arg("-c").arg(&clean).spawn();
                        return cosmic::iced::exit();
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

    pub fn view(&self) -> Element<'_, Message> {
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
        crate::ui::panels::compose(
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
        )
    }

    pub fn theme(_: &Self) -> Theme {
        Theme::Dark
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            event::listen().map(Message::WindowEvent),
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
            soulless_organizer::subscription().map(Message::Organizer),
            crate::network_monitor::subscription().map(Message::Network),
            crate::system_monitor::subscription().map(Message::System),
            crate::hardware_monitor::subscription().map(Message::Hardware),
            crate::fps_monitor::subscription().map(Message::Fps),
        ])
    }
}