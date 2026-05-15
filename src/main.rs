// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

use cosmic::iced::keyboard::key::Named;
use cosmic::iced::{
    Element, Length, Subscription, Task, Theme, event, keyboard, widget::container, window,
};
use fs2::FileExt;
use std::fs::OpenOptions;
use std::path::PathBuf;

mod drawers;
mod position;
mod search;

use position::LauncherPosition;
use search::Message as SearchMessage;

pub enum Message {
    Search(SearchMessage),
    WindowEvent(cosmic::iced::Event),
}

fn main() -> cosmic::iced::Result {
    if !ensure_single_instance() {
        eprintln!("Soulless is already running.");
        return Ok(());
    }

    let position = LauncherPosition;

    cosmic::iced::application(Soulless::new, Soulless::update, Soulless::view)
        .subscription(Soulless::subscription)
        .window_size(position.window_size())
        .position(window::Position::Specific(position.window_position()))
        .decorations(false)
        .transparent(true)
        .resizable(false)
        .theme(Soulless::theme)
        .run()
}

struct Soulless {
    search: search::Search,
}

impl Soulless {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                search: search::Search::new(),
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

                    cosmic::iced::exit()
                } else {
                    Task::none()
                }
            }

            Message::WindowEvent(cosmic::iced::Event::Keyboard(
                keyboard::Event::KeyPressed { key, .. },
            )) => {
                if matches!(key, keyboard::Key::Named(Named::Escape)) {
                    cosmic::iced::exit()
                } else {
                    Task::none()
                }
            }

            _ => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = drawers::view(&self.search).map(Message::Search);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(cosmic::iced::Color::from_rgb8(30, 30, 30).into()),
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

// === DONE ===
// Removed unsupported text_input::Id API :: done
// Removed unsupported text_input::focus API :: done
// Removed unsupported FocusSearchBar message :: done
// Launcher still exits after app launch :: done
// transparent(true) retained for Wayland rounded corners :: done
// Single-instance lock retained :: done