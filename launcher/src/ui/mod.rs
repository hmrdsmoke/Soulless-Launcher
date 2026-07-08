// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/ui/mod.rs
// Launcher UI module: startup blur task and submodule re-exports.

pub mod theme;
pub mod panels;
pub mod widgets;

/// Visual startup tasks — run once when the window opens.
/// Enables compositor-level background blur for the glassmorphism effect.
pub fn startup_tasks<Message: 'static + Send>(id: cosmic::iced::window::Id) -> cosmic::iced::Task<Message> {
    cosmic::iced::Task::batch([
        cosmic::iced::window::enable_blur(id),
        cosmic::iced::window::gain_focus(id),
        cosmic::widget::text_input::focus(cosmic::widget::Id::new("soulless-search-bar")),
    ])
}
pub mod organizer;
