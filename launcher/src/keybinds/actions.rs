// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// src/keybinds/actions.rs
// Defines what each key does in the launcher.

use crate::search;
use cosmic::iced::Task;
use cosmic::iced::keyboard::{self, key::Named};

/// Scroll the results view so the focused item stays visible. One discrete
/// snap per keypress (no continuous re-render). Called only from explicit
/// arrow-key handlers, so hover (which routes elsewhere) never triggers it.
fn scroll_to_focused<Message: 'static>(search: &search::Search) -> Task<Message> {
    // The pixel offset lives on Search, which owns the grid geometry
    // (ROW_H / GRID_COLUMNS at search module scope).
    if let Some(y) = search.focused_scroll_offset() {
        return cosmic::iced::widget::scrollable::scroll_to(
            cosmic::widget::Id::new("soulless-results-scroll"),
            cosmic::iced::widget::scrollable::AbsoluteOffset { x: Some(0.0), y: Some(y) },
        );
    }
    Task::none()
}

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

        // ── ArrowDown → down one grid ROW in results, else next drawer ────
        keyboard::Key::Named(Named::ArrowDown) => {
            // In the results grid, Down moves a full row (+GRID_COLUMNS).
            // Left/Right own the ±1 moves; Down/Up own the vertical.
            if search.show_search_results {
                let len = search.current_grid_len();
                if len > 0 {
                    let cols = search::GRID_COLUMNS;
                    let next_idx = match search.focused_app_idx {
                        None => 0,
                        Some(i) if i + cols < len => i + cols,
                        // Above a partial last row with no tile directly
                        // below: drop to the last item instead of sticking.
                        Some(i) if i / cols < (len - 1) / cols => len - 1,
                        Some(i) => i, // already in the last row, stay
                    };
                    search.update(search::Message::FocusApp(next_idx));
                }
                return scroll_to_focused(search);
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

        // ── ArrowUp → up one grid ROW (top row returns to search), else prev drawer ─
        keyboard::Key::Named(Named::ArrowUp) => {
            if search.show_search_results {
                let cols = search::GRID_COLUMNS;
                match search.focused_app_idx {
                    Some(i) if i >= cols => {
                        search.update(search::Message::FocusApp(i - cols));
                        return scroll_to_focused(search);
                    }
                    _ => {
                        // Anywhere in the top row (or unfocused) — return
                        // focus to the search bar.
                        search.update(search::Message::ClearFocus);
                        return cosmic::widget::text_input::focus(
                            cosmic::widget::Id::new("soulless-search-bar")
                        );
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
            if let Some(idx) = search.focused_app_idx
                && let Some(exec) = search.focused_exec(idx) {
                    search.record_launch_by_exec(&exec);
                    // Shared launch path: strips placeholders, routes through
                    // flatpak-spawn --host when sandboxed, and reaps the child
                    // (raw spawn here leaked one zombie per Enter launch).
                    crate::utils::spawn_exec(&exec);
                    return f_exit();
                }
            Task::none()
        }

        // ── ArrowRight → next app in grid ────────────────────────────────
        keyboard::Key::Named(Named::ArrowRight) => {
            search.update(search::Message::FocusNext);
            scroll_to_focused(search)
        }

        // ── ArrowLeft → prev app in grid ─────────────────────────────────
        keyboard::Key::Named(Named::ArrowLeft) => {
            search.update(search::Message::FocusPrev);
            scroll_to_focused(search)
        }

        // ── Ctrl+1-9 → jump directly to drawer by index ─────────────────
        keyboard::Key::Character(c) if modifiers.control() => {
            if let Ok(n) = c.as_str().parse::<usize>()
                && n >= 1 {
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
// Ctrl+1-9 jump to drawer by index :: done #19
// ArrowUp/ArrowDown move by grid row in results (±GRID_COLUMNS) :: done
// === PLANNED ===
// ArrowDown from last drawer → enter app grid :: see issue #17