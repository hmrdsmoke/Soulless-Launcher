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
    modifiers: keyboard::Modifiers,
    search: &mut search::Search,
    _f_search_msg: impl Fn(search::Message) -> Message + Copy,
    f_exit: impl FnOnce() -> Task<Message>,
) -> Task<Message>
where
    Message: 'static + Send + Clone,
{
    // Single-character shortcuts only fire when not in search/typing mode
    let in_search = matches!(search.current_open_drawer, search::OpenDrawer::Search);

    match key {
        // ── Esc → close launcher ─────────────────────────────────────────
        keyboard::Key::Named(Named::Escape) => f_exit(),

        // ── Tab → cycle to next drawer, Shift+Tab passes through ─────────
        keyboard::Key::Named(Named::Tab) => {
            if modifiers.shift() {
                return Task::none();
            }
            let next_name = {
                let drawers = search.drawer_state.drawers();
                if drawers.is_empty() {
                    return Task::none();
                }
                let current_idx = if let search::OpenDrawer::Pinned(name) = &search.current_open_drawer {
                    drawers.iter().position(|d| &d.name == name)
                } else {
                    None
                };
                let next_idx = current_idx.map(|i| (i + 1) % drawers.len()).unwrap_or(0);
                drawers[next_idx].name.clone()
            };
            search.update(search::Message::DrawerClicked(next_name.clone()));
            cosmic::widget::button::focus(
                cosmic::widget::Id::new(format!("drawer-btn-{}", next_name))
            )
        }

        // ── ArrowDown → next drawer ───────────────────────────────────────
        keyboard::Key::Named(Named::ArrowDown) => {
            let next = {
                let drawers = search.drawer_state.drawers();
                if let search::OpenDrawer::Pinned(name) = &search.current_open_drawer {
                    let idx = drawers.iter().position(|d| &d.name == name).unwrap_or(0);
                    if idx + 1 < drawers.len() {
                        Some(drawers[idx + 1].name.clone())
                    } else {
                        None
                    }
                } else {
                    drawers.first().map(|d| d.name.clone())
                }
            };
            if let Some(name) = next {
                search.update(search::Message::DrawerClicked(name.clone()));
                return cosmic::widget::button::focus(
                    cosmic::widget::Id::new(format!("drawer-btn-{}", name))
                );
            }
            Task::none()
        }

        // ── ArrowUp → previous drawer ─────────────────────────────────────
        keyboard::Key::Named(Named::ArrowUp) => {
            let prev = {
                let drawers = search.drawer_state.drawers();
                if let search::OpenDrawer::Pinned(name) = &search.current_open_drawer {
                    let idx = drawers.iter().position(|d| &d.name == name).unwrap_or(0);
                    if idx > 0 {
                        Some(drawers[idx - 1].name.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            };
            if let Some(name) = prev {
                search.update(search::Message::DrawerClicked(name.clone()));
                return cosmic::widget::button::focus(
                    cosmic::widget::Id::new(format!("drawer-btn-{}", name))
                );
            }
            Task::none()
        }

        // ── S → focus search bar (only when not already in search) ────────
        keyboard::Key::Character(c) if c.as_str() == "s" && !in_search => {
            search.update(search::Message::QueryChanged(String::new()));
            cosmic::widget::text_input::focus(
                cosmic::widget::Id::new("soulless-search-bar")
            )
        }

        // ── V → open vault (only when not in search/typing mode) ─────────
        keyboard::Key::Character(c) if c.as_str() == "v" && !in_search => {
            search.update(search::Message::VaultClicked);
            Task::none()
        }

        _ => Task::none(),
    }
}

// === DONE ===
// Escape closes launcher :: done
// Tab cycles drawer buttons :: done
// ArrowUp/ArrowDown navigate drawers :: done
// S focuses search bar :: done
// V opens vault :: done
// === PLANNED ===
// ArrowLeft/ArrowRight navigate app grid :: see issue #16
// ArrowDown from last drawer → enter app grid :: see issue #17
// Enter launches selected app :: see issue #18
// Ctrl+1-4 jump to drawer :: see issue #19
