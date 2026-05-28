// MIT License - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// src/keybinds/actions.rs
// Defines what each key does in the launcher.

use crate::search;
use cosmic::iced::Task;
use cosmic::iced::keyboard::{self, key::Named};

/// Handle a key press and return the appropriate task.
pub fn handle_key<Message>(
    key: &keyboard::Key,
    search: &mut search::Search,
    _f_search_msg: impl Fn(search::Message) -> Message + Copy,
    f_exit: impl FnOnce() -> Task<Message>,
    f_focus_search: impl FnOnce() -> Task<Message>,
) -> Task<Message>
where
    Message: 'static + Send + Clone,
{
    match key {
        // ── Esc → close launcher ──────────────────────────────────────────
        keyboard::Key::Named(Named::Escape) => f_exit(),

        // ── Tab → focus first drawer button ──────────────────────────────
        keyboard::Key::Named(Named::Tab) => {
            if let Some(first) = search.drawer_state.drawers().first() {
                let name = first.name.clone();
                search.update(search::Message::DrawerClicked(name.clone()));
                return cosmic::widget::button::focus(
                    cosmic::widget::Id::new(format!("drawer-btn-{}", name))
                );
            }
            Task::none()
        }

        // ── Shift+Tab → focus search bar ──────────────────────────────────
        // (handled by the search bar's own focus logic)

        // ── Any printable key → ensure search bar is focused ──────────────
        _ => f_focus_search(),
    }
}

// === DONE ===
// Escape closes launcher :: done
// Tab opens first drawer :: done
// Any other key refocuses search bar :: done
// === IN PROGRESS ===
// Shift+Tab back to search bar :: in progress
// Up/Down arrow navigate results :: in progress
// Enter launches selected app :: in progress
// Ctrl+1-4 jump to drawer :: planned
