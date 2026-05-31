// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.


use cosmic::iced::{
    Element, Subscription, Task, Theme,
    event, keyboard,
    window,
};
use cosmic::iced::clipboard::dnd::{DndEvent, OfferEvent};
use fs2::FileExt;
use std::fs::OpenOptions;
use std::path::PathBuf;

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
mod utils;


use position::LauncherPosition;
use search::Message as SearchMessage;
use network_monitor::Message as NetworkMessage;
use system_monitor::Message as SystemMessage;
use hardware_monitor::Message as HardwareMessage;
use fps_monitor::Message as FpsMessage;

// ── Top-level message ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Message {
    Search(SearchMessage),
    Network(NetworkMessage),
    System(SystemMessage),
    Hardware(HardwareMessage),
    Fps(FpsMessage),
    WindowEvent(cosmic::iced::Event),
}

// ── Application model ────────────────────────────────────────────────────────

struct Soulless {
    search:     search::Search,
    network:    network_monitor::NetworkState,
    system:     system_monitor::SystemState,
    hardware:   hardware_monitor::HardwareMonitorState,
    fps:        fps_monitor::FpsMonitorState,
    cursor_pos: Option<cosmic::iced::Point>,
    bg_handle:  Option<cosmic::iced::widget::image::Handle>,
}

impl Soulless {
    fn new() -> (Self, Task<Message>) {
        let bg_handle = config::default_background().map(|path| {
            let width = position::layout::RIGHT_PANEL_WIDTH as u32;
            if let Some(rgba) = config::load_background_rgba(&path, width, 900) {
                cosmic::iced::widget::image::Handle::from_rgba(width, 900, rgba)
            } else {
                cosmic::iced::widget::image::Handle::from_path(path)
            }
        });

        (
            Self {
                search:     search::Search::new(),
                network:    network_monitor::NetworkState::new(),
                system:     system_monitor::SystemState::new(),
                hardware:   hardware_monitor::HardwareMonitorState::new(),
                fps:        fps_monitor::FpsMonitorState::new(),
                cursor_pos: None,
                bg_handle,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
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
                    let clean_exec = utils::strip_desktop_placeholders(&exec);

                    if let Err(e) = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&clean_exec)
                        .spawn()
                    {
                        eprintln!("Failed to launch app: {}", e);
                    }

                    cosmic::iced::exit::<Message>()
                } else {
                    Task::none()
                }
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
                return keybinds::actions::handle_key(
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
                use search::OpenDrawer;

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
                                let payload = String::from_utf8_lossy(&data);

                                let paths: Vec<PathBuf> = payload
                                    .lines()
                                    .map(str::trim)
                                    .filter(|l| l.starts_with("file://"))
                                    .filter_map(|l| {
                                        let raw =
                                            l.trim_start_matches("file://");
                                        let decoded = utils::percent_decode_uri(raw);
                                        let p = PathBuf::from(decoded);
                                        if p.exists() { Some(p) } else { None }
                                    })
                                    .collect();

                                if !paths.is_empty() {
                                    self.search.update(
                                        SearchMessage::VaultFilesDropped(paths),
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

            _ => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let (toolbox, right) = drawers::view(&self.search);
        ui::panels::compose(
            toolbox.map(Message::Search),
            right.map(Message::Search),
            network_monitor::view(&self.network).map(Message::Network),
            system_monitor::view(&self.system).map(Message::System),
            hardware_monitor::view(&self.hardware).map(Message::Hardware),
            fps_monitor::view(&self.fps).map(Message::Fps),
            self.bg_handle.clone(),
        )
    }

    fn theme(_: &Self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            event::listen().map(Message::WindowEvent),
            network_monitor::subscription().map(Message::Network),
            system_monitor::subscription().map(Message::System),
            hardware_monitor::subscription().map(Message::Hardware),
            fps_monitor::subscription().map(Message::Fps),
        ])
    }
}

// ── Entry point ──────────────────────────────────────────────────────────────

fn main() -> cosmic::iced::Result {
    if !ensure_single_instance() {
        eprintln!("Soulless is already running.");
        return Ok(());
    }

    let position = LauncherPosition;

    cosmic::iced::application(
        Soulless::new,
        Soulless::update,
        Soulless::view,
    )
    .subscription(Soulless::subscription)
    .theme(Soulless::theme)
    .window_size(position.window_size())
    .position(window::Position::Specific(
        position.window_position(),
    ))
    .decorations(false)
    .transparent(true)
    .resizable(false)
    .run()
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn ensure_single_instance() -> bool {
    let lock_path = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("soulless/soulless.lock");

    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    if let Ok(file) = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&lock_path)
    {
        if file.try_lock_exclusive().is_ok() {
            #[allow(clippy::mem_forget)]
            Box::leak(Box::new(file));

            return true;
        }
    }

    false
}
