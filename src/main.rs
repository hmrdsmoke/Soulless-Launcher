use cosmic::iced::keyboard::key::Named;
use cosmic::iced::{
    Element, Length, Subscription, Task, Theme,
    event, keyboard,
    widget::container,
    window,
};
use cosmic::iced::clipboard::dnd::{DndEvent, OfferEvent};
use fs2::FileExt;
use std::fs::OpenOptions;
use std::path::PathBuf;

mod drawers;
mod drawers_state;
mod indexer;
mod position;
mod search;
mod vault;
mod vault_ui;

use position::LauncherPosition;
use search::Message as SearchMessage;

// ── Top-level message ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Message {
    Search(SearchMessage),
    WindowEvent(cosmic::iced::Event),
}

// ── Application model ─────────────────────────────────────────

struct Soulless {
    search: search::Search,
    cursor_pos: Option<cosmic::iced::Point>,
}

impl Soulless {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                search: search::Search::new(),
                cursor_pos: None,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Search(msg) => {
                if let Some(exec) = self.search.update(msg) {
                    let clean_exec = strip_desktop_placeholders(&exec);

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

            // ── Keyboard ──────────────────────────────────────────────────────
            Message::WindowEvent(
                cosmic::iced::Event::Keyboard(
                    keyboard::Event::KeyPressed { key, .. },
                ),
            ) => {
                if matches!(key, keyboard::Key::Named(Named::Escape)) {
                    return cosmic::iced::exit::<Message>();
                }
                Task::none()
            }

            // ── Track cursor position ────────────────────────────────────────
            Message::WindowEvent(cosmic::iced::Event::Mouse(
                cosmic::iced::mouse::Event::CursorMoved { position },
            )) => {
                self.cursor_pos = Some(position);
                Task::none()
            }

            // ── Click outside → exit ──────────────────────────────────────────
            Message::WindowEvent(cosmic::iced::Event::Mouse(
                cosmic::iced::mouse::Event::ButtonPressed(_),
            )) => {
                // Exit if the click landed outside the window bounds.
                // window_size() returns (width, height) as f32 via Size.
                let size = LauncherPosition.window_size();
                let outside = self.cursor_pos.map_or(false, |p| {
                    p.x < 0.0 || p.y < 0.0
                        || p.x > size.width
                        || p.y > size.height
                });
                if outside {
                    return cosmic::iced::exit::<Message>();
                }
                Task::none()
            }

            // ── Drag-and-drop ─────────────────────────────────────────────────
            Message::WindowEvent(cosmic::iced::Event::Dnd(dnd_event)) => {
                use search::OpenDrawer;

                if self.search.current_open_drawer != OpenDrawer::Vault {
                    return Task::none();
                }

                match dnd_event {
                    DndEvent::Offer(_, OfferEvent::Enter { .. }) => {
                        self.search.update(SearchMessage::VaultDragHover(true));
                    }

                    DndEvent::Offer(_, OfferEvent::Leave)
                    | DndEvent::Offer(_, OfferEvent::LeaveDestination) => {
                        self.search.update(SearchMessage::VaultDragHover(false));
                    }

                    DndEvent::Offer(_, OfferEvent::Data { data, mime_type }) => {
                        if mime_type == "text/uri-list" {
                            let payload = String::from_utf8_lossy(&data);
                            let paths: Vec<PathBuf> = payload
                                .lines()
                                .map(str::trim)
                                .filter(|l| l.starts_with("file://"))
                                .filter_map(|l| {
                                    let raw = l.trim_start_matches("file://");
                                    let decoded = percent_decode_uri(raw);
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
                        self.search.update(SearchMessage::VaultDragHover(false));
                    }

                    _ => {}
                }

                Task::none()
            }

            _ => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = drawers::view(&self.search)
            .map(Message::Search);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(
                    cosmic::iced::Color::from_rgb8(30, 30, 30).into(),
                ),
                border: cosmic::iced::border::rounded(8),
                ..Default::default()
            })
            .into()
    }

    fn theme(_: &Self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::WindowEvent)
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

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

// ── Helpers ───────────────────────────────────────────────────────────────────

fn strip_desktop_placeholders(exec: &str) -> String {
    let mut result = String::with_capacity(exec.len());
    let mut chars = exec.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            if chars
                .peek()
                .map_or(false, |&next| next.is_ascii_alphabetic())
            {
                chars.next();
                continue;
            }
        }
        result.push(c);
    }

    result.trim().to_string()
}

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

// ── URI percent-decoding ──────────────────────────────────────────────────────

fn percent_decode_uri(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (
                hex_nibble(bytes[i + 1]),
                hex_nibble(bytes[i + 2]),
            ) {
                out.push((hi << 4 | lo) as char);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }

    out
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

// === Done ===
// Added superkey opens 
// added press outside closes

// === DONE ===
// Added Event::Dnd handling in update() via the existing subscription :: done
// OfferEvent::Enter  → VaultDragHover(true)  :: done
// OfferEvent::Leave / LeaveDestination → VaultDragHover(false) :: done
// OfferEvent::Data   → parse text/uri-list → VaultFilesDropped :: done
// Only fires when current_open_drawer == Vault :: done
// percent_decode_uri handles %20 and other encoded path chars :: done
// dnd_destination widget removed — no cosmic::Theme conflict :: done
// All other update arms unchanged :: done

// === DONE ===
// Switched back to cosmic::iced::application() for full window control :: done
// decorations(false) — protocol-level request to compositor: no decorations :: done
// resizable(false) — no resize border :: done
// transparent(true) — rounded corners on Wayland :: done
// Theme::Dark preserved :: done
// Single-instance lock preserved :: done
// strip_desktop_placeholders preserved :: done

// === DONE ===
// client_decorations(false) — removes title bar and window buttons :: done
// resizable(None) — disables resize border entirely :: done
// transparent(true) — keeps rounded corners on Wayland :: done
// exit_on_close(false) — exit controlled by Escape/launch, not window X :: done
// is_daemon(false) — foreground launcher window :: done
// cosmic::app::run() — uses wgpu renderer on Wayland :: done
// Single-instance lock preserved :: done
// strip_desktop_placeholders preserved :: done

// === DONE ===
// Replaced cosmic::iced::application() with cosmic::Application trait :: done
// Now uses cosmic::app::run() which selects wgpu on Wayland :: done
// cosmic::app::Settings replaces the chained builder :: done
// KeyPressed handling preserved via event::listen_with :: done
// Single-instance lock preserved :: done
// strip_desktop_placeholders preserved :: done

// === DONE ===
// Removed unsupported text_input::Id API :: done
// Removed unsupported text_input::focus API :: done
// Removed unsupported FocusSearchBar message :: done
// Launcher still exits after app launch :: done
// transparent(true) retained for Wayland rounded corners :: done
// Single-instance lock retained :: done