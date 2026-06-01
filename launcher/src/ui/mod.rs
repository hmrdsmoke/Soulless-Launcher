// MIT License - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// src/ui/mod.rs
pub mod theme;
pub mod panels;
pub mod widgets;

/// Visual startup tasks — run once when the window opens.
/// Enables compositor-level background blur for the glassmorphism effect.
pub fn startup_tasks<Message: 'static + Send>(id: cosmic::iced::window::Id) -> cosmic::iced::Task<Message> {
    cosmic::iced::window::enable_blur(id)
}
pub mod organizer;
