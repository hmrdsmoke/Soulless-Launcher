// GPL-3.0-or-later - see LICENSE file for full terms
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

        // ── ArrowDown → into search results, else next drawer ─────────────
        keyboard::Key::Named(Named::ArrowDown) => {
            // If search results are showing, Arrow Down navigates into/through
            // them instead of jumping to drawers.
            if search.show_search_results {
                let len = search.current_grid_len();
                if len > 0 {
                    let next_idx = match search.focused_app_idx {
                        None => 0,
                        Some(i) if i + 1 < len => i + 1,
                        Some(i) => i, // already at last result, stay
                    };
                    search.update(search::Message::FocusApp(next_idx));
                }
                return Task::none();
            }
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

        // ── ArrowUp → up through results (top returns to search), else prev drawer ─
        keyboard::Key::Named(Named::ArrowUp) => {
            if search.show_search_results {
                match search.focused_app_idx {
                    Some(0) | None => {
                        // At the top of results — return focus to the search bar.
                        search.update(search::Message::ClearFocus);
                        return cosmic::widget::text_input::focus(
                            cosmic::widget::Id::new("soulless-search-bar")
                        );
                    }
                    Some(i) => {
                        search.update(search::Message::FocusApp(i - 1));
                        return Task::none();
                    }
                }
            }
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

        // ── Enter → launch focused app ───────────────────────────────────
        keyboard::Key::Named(Named::Enter) => {
            if let Some(idx) = search.focused_app_idx {
                if let Some(exec) = search.focused_exec(idx) {
                    search.record_launch_by_exec(&exec);
                    let clean = crate::utils::strip_desktop_placeholders(&exec);
                    let _ = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&clean)
                        .spawn();
                    return f_exit();
                }
            }
            Task::none()
        }

        // ── ArrowRight → next app in grid ────────────────────────────────
        keyboard::Key::Named(Named::ArrowRight) => {
            search.update(search::Message::FocusNext);
            Task::none()
        }

        // ── ArrowLeft → prev app in grid ─────────────────────────────────
        keyboard::Key::Named(Named::ArrowLeft) => {
            search.update(search::Message::FocusPrev);
            Task::none()
        }

        // ── Ctrl+1-9 → jump directly to drawer by index ─────────────────
        keyboard::Key::Character(c) if modifiers.control() => {
            if let Ok(n) = c.as_str().parse::<usize>() {
                if n >= 1 {
                    let idx = n - 1;
                    let name = {
                        let drawers = search.drawer_state.drawers();
                        drawers.get(idx).map(|d| d.name.clone())
                    };
                    if let Some(name) = name {
                        search.update(search::Message::DrawerClicked(name.clone()));
                        return cosmic::widget::button::focus(
                            cosmic::widget::Id::new(format!("drawer-btn-{}", name))
                        );
                    }
                }
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
            cosmic::widget::text_input::focus(
                cosmic::widget::Id::new("vault-password")
            )
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
// ArrowLeft/ArrowRight navigate app grid :: done #16
// Enter launches focused app :: done #18
// === PLANNED ===
// ArrowDown from last drawer → enter app grid :: see issue #17
// Ctrl+1-9 jump to drawer by index :: done #19
