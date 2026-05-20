// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

use crate::vault::Vault;
use crate::drawers_state::DrawerState;
use crate::indexer::{build_index, AppEntry};

use nucleo_matcher::pattern::{
    AtomKind,
    CaseMatching,
    Normalization,
    Pattern,
};

use nucleo_matcher::{Config, Matcher};

use std::path::PathBuf;

// ── Messages ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Message {
    QueryChanged(String),
    AppClicked(String),

    DrawerClicked(String),
    VaultClicked,
    SearchBarClicked,

    // Context menu
    RightClickDrawerBackground(String),
    RightClickDrawerSidebar(String),
    RightClickDrawerApp(String, String),
    CloseContextMenu,

    // App picker
    OpenAppPicker(String),
    AppPickerQueryChanged(String),
    AddAppToDrawer(String, String),
    RemoveAppFromDrawer(String, String),
    ClearDrawer(String),
    CloseAppPicker,

    // Drawer management
    CreateDrawer,
    DeleteDrawer(String),
    MoveDrawerUp(String),
    MoveDrawerDown(String),
    RenameDrawer(String, String),

    // Vault
    VaultPasswordChanged(String),
    VaultConfirmChanged(String),

    VaultSetupConfirm,
    VaultUnlock,
    VaultLock,

    VaultOpenFile(String),
    VaultRemoveFile(String),

    VaultOpenFileMenu(String),

    /// Fired when files are dropped onto the vault drop zone
    VaultFilesDropped(Vec<std::path::PathBuf>),
    /// Fired on drag enter/leave to toggle the hover highlight
    VaultDragHover(bool),

    /// Fired when an app is dragged over a sidebar drawer (Some) or leaves (None)
    DrawerDragHover(Option<String>),
    /// Fired when an app icon is dropped onto a drawer in the sidebar
    AppDroppedOnDrawer(String, String), // (drawer_name, app_id)
}

// ── Drawer state ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenDrawer {
    Search,
    Pinned(String),
    Vault,
}

#[derive(Debug, Clone)]
pub enum ContextMenu {
    DrawerBackground {
        drawer: String,
    },

    DrawerSidebar {
        drawer: String,
    },

    DrawerApp {
        drawer: String,
        app_id: String,
    },
}

// ── Search model ──────────────────────────────────────────────────────────────

pub struct Search {
    pub query: String,

    matcher: Matcher,

    pub all_apps: Vec<AppEntry>,

    filtered_apps: Vec<usize>,

    pub show_search_results: bool,

    pub current_open_drawer: OpenDrawer,

    pub drawer_state: DrawerState,

    pub context_menu: Option<ContextMenu>,

    pub app_picker: Option<AppPicker>,

    pub vault: Vault,

    /// Which sidebar drawer is currently being hovered by a drag
    pub drag_hover_drawer: Option<String>,
}

pub struct AppPicker {
    pub drawer: String,
    pub query: String,
    pub filtered: Vec<usize>,
}

// ── Search impl ───────────────────────────────────────────────────────────────

impl Search {
    pub fn new() -> Self {
        let matcher = Matcher::new(Config::DEFAULT);

        let all_apps = build_index();

        let drawer_state =
            load_drawer_state().unwrap_or_default();

        let mut search = Self {
            query: String::new(),

            matcher,

            all_apps,

            filtered_apps: Vec::new(),

            show_search_results: true,

            current_open_drawer:
                OpenDrawer::Search,

            drawer_state,

            context_menu: None,

            app_picker: None,

            vault: Vault::new(),

            drag_hover_drawer: None,
        };

        search.recompute_results();

        search
    }

    pub fn update(
        &mut self,
        message: Message,
    ) -> Option<String> {
        match message {
            // ── Search ─────────────────────────────

            Message::QueryChanged(q) => {
                self.query = q;

                self.recompute_results();

                self.show_search_results = true;

                self.current_open_drawer =
                    OpenDrawer::Search;

                self.context_menu = None;

                None
            }

            Message::AppClicked(exec) => {
                Some(exec)
            }

            Message::DrawerClicked(name) => {
                self.current_open_drawer =
                    OpenDrawer::Pinned(name);

                self.show_search_results = false;

                self.context_menu = None;

                self.app_picker = None;

                None
            }

            Message::VaultClicked => {
                self.current_open_drawer =
                    OpenDrawer::Vault;

                self.show_search_results = false;

                self.context_menu = None;

                None
            }

            Message::SearchBarClicked => {
                self.show_search_results = true;

                self.current_open_drawer =
                    OpenDrawer::Search;

                self.context_menu = None;

                None
            }

            // ── Context menus ─────────────────────

            Message::RightClickDrawerBackground(
                drawer,
            ) => {
                self.context_menu = Some(
                    ContextMenu::DrawerBackground {
                        drawer,
                    },
                );

                None
            }

            Message::RightClickDrawerSidebar(
                drawer,
            ) => {
                self.context_menu = Some(
                    ContextMenu::DrawerSidebar {
                        drawer,
                    },
                );

                None
            }

            Message::RightClickDrawerApp(
                drawer,
                app_id,
            ) => {
                self.context_menu =
                    Some(ContextMenu::DrawerApp {
                        drawer,
                        app_id,
                    });

                None
            }

            Message::CloseContextMenu => {
                self.context_menu = None;

                None
            }

            // ── App picker ────────────────────────

            Message::OpenAppPicker(drawer) => {
                let mut picker = AppPicker {
                    drawer,

                    query: String::new(),

                    filtered: Vec::new(),
                };

                picker.filtered =
                    (0..self.all_apps.len())
                        .collect();

                self.app_picker =
                    Some(picker);

                self.context_menu = None;

                None
            }

            Message::AppPickerQueryChanged(q) => {
                if let Some(picker) =
                    &mut self.app_picker
                {
                    picker.query = q.clone();

                    let q_lower =
                        q.to_lowercase();

                    picker.filtered = self
                        .all_apps
                        .iter()
                        .enumerate()
                        .filter(|(_, app)| {
                            app.lower_name
                                .contains(
                                    &q_lower
                                )
                        })
                        .map(|(i, _)| i)
                        .collect();
                }

                None
            }

            Message::CloseAppPicker => {
                self.app_picker = None;

                None
            }

            // ── Drawer management ─────────────────

            Message::CreateDrawer => {
                let mut index = 1;

                loop {
                    let name = format!(
                        "New Drawer {}",
                        index
                    );

                    let exists = self
                        .drawer_state
                        .drawers()
                        .iter()
                        .any(|d| {
                            d.name == name
                        });

                    if !exists {
                        self.drawer_state
                            .create_drawer(
                                name.clone(),
                                "📁"
                                    .to_string(),
                            );

                        save_drawer_state(
                            &self.drawer_state,
                        );

                        break;
                    }

                    index += 1;
                }

                None
            }

            Message::DeleteDrawer(name) => {
                self.drawer_state
                    .remove_drawer(&name);

                save_drawer_state(
                    &self.drawer_state,
                );

                self.current_open_drawer =
                    OpenDrawer::Search;

                self.context_menu = None;

                None
            }

            Message::MoveDrawerUp(name) => {
                self.drawer_state
                    .move_drawer_up(&name);

                save_drawer_state(
                    &self.drawer_state,
                );

                self.context_menu = None;

                None
            }

            Message::MoveDrawerDown(name) => {
                self.drawer_state
                    .move_drawer_down(&name);

                save_drawer_state(
                    &self.drawer_state,
                );

                self.context_menu = None;

                None
            }

            Message::RenameDrawer(
                old,
                new,
            ) => {
                self.drawer_state
                    .rename_drawer(
                        &old,
                        &new,
                    );

                save_drawer_state(
                    &self.drawer_state,
                );

                self.context_menu = None;

                if self.current_open_drawer
                    == OpenDrawer::Pinned(
                        old.clone(),
                    )
                {
                    self.current_open_drawer =
                        OpenDrawer::Pinned(new);
                }

                None
            }

            // ── Drawer apps ───────────────────────

            Message::AddAppToDrawer(
                drawer,
                app_id,
            ) => {
                self.drawer_state.add_app(
                    &drawer,
                    app_id,
                );

                save_drawer_state(
                    &self.drawer_state,
                );

                None
            }

            Message::RemoveAppFromDrawer(
                drawer,
                app_id,
            ) => {
                self.drawer_state
                    .remove_app(
                        &drawer,
                        &app_id,
                    );

                save_drawer_state(
                    &self.drawer_state,
                );

                self.context_menu = None;

                None
            }

            Message::ClearDrawer(drawer) => {
                self.drawer_state
                    .clear_drawer(
                        &drawer,
                    );

                save_drawer_state(
                    &self.drawer_state,
                );

                self.context_menu = None;

                None
            }

            // ── Vault ─────────────────────────────

            Message::VaultPasswordChanged(
                value,
            ) => {
                self.vault.password_input =
                    value;
                self.vault.error = None;

                None
            }

            Message::VaultConfirmChanged(
                value,
            ) => {
                self.vault.confirm_input =
                    value;
                self.vault.error = None;

                None
            }

            Message::VaultSetupConfirm => {
                self.vault.finish_setup();

                None
            }

            Message::VaultUnlock => {
                self.vault.unlock();

                None
            }

            Message::VaultLock => {
                self.vault.lock();

                None
            }

            Message::VaultOpenFile(id) => {
                if let Err(e) =
                    self.vault.open_file(&id)
                {
                    self.vault.error = Some(e);
                }

                None
            }

            Message::VaultRemoveFile(id) => {
                if let Err(e) =
                    self.vault.remove_file(&id)
                {
                    self.vault.error = Some(e);
                }

                None
            }

            Message::VaultOpenFileMenu(_) => {
                None
            }

            Message::VaultDragHover(hovering) => {
                self.vault.drag_hover = hovering;
                None
            }

            Message::DrawerDragHover(drawer) => {
                self.drag_hover_drawer = drawer;
                None
            }

            Message::AppDroppedOnDrawer(drawer, app_id) => {
                self.drag_hover_drawer = None;
                self.drawer_state.add_app(&drawer, app_id);
                save_drawer_state(&self.drawer_state);
                None
            }

            Message::VaultFilesDropped(paths) => {
                self.vault.error = None;
                self.vault.status = None;

                let mut added = 0usize;
                let mut last_error: Option<String> = None;

                for path in &paths {
                    match self.vault.add_file(path) {
                        Ok(()) => {
                            added += 1;
                        }
                        Err(e) if e.starts_with("NEEDS_PKEXEC:") => {
                            // Encrypted copy succeeded; original removal needs elevation.
                            // Count as added — the file is safe in the vault.
                            added += 1;
                            eprintln!("pkexec needed to remove original: {e}");
                        }
                        Err(e) => {
                            last_error = Some(e);
                        }
                    }
                }

                self.vault.drag_hover = false;

                if added > 0 {
                    self.vault.status = Some(match added {
                        1 => "1 file added to vault.".to_string(),
                        n => format!("{n} files added to vault."),
                    });
                }

                if let Some(e) = last_error {
                    self.vault.error = Some(e);
                }

                None
            }
        }
    }

    // ── Accessors ───────────────────────────────

    pub fn filtered_apps(
        &self,
    ) -> &[usize] {
        &self.filtered_apps
    }

    pub fn app(
        &self,
        index: usize,
    ) -> Option<&AppEntry> {
        self.all_apps.get(index)
    }

    pub fn app_by_id(
        &self,
        id: &str,
    ) -> Option<&AppEntry> {
        self.all_apps
            .iter()
            .find(|a| a.id == id)
    }

    pub fn drawers(&self) -> Vec<String> {
        self.drawer_state.drawer_names()
    }

    // ── Search internals ────────────────────────

    fn recompute_results(&mut self) {
        const MAX_RESULTS: usize = 200;
        const TOP_PREFIX_COUNT: usize = 12;

        if self.query.is_empty() {
            self.filtered_apps =
                (0..self
                    .all_apps
                    .len()
                    .min(MAX_RESULTS))
                    .collect();

            return;
        }

        let query_lower =
            self.query.to_lowercase();

        let char_count =
            self.query.chars().count();

        let prefix_indices: Vec<usize> =
            self
                .all_apps
                .iter()
                .enumerate()
                .filter(|(_, app)| {
                    app.lower_name
                        .starts_with(
                            &query_lower
                        )
                })
                .map(|(i, _)| i)
                .collect();

        let top12: Vec<usize> =
            prefix_indices
                .iter()
                .copied()
                .take(TOP_PREFIX_COUNT)
                .collect();

        let remaining_budget =
            MAX_RESULTS
                .saturating_sub(top12.len());

        match char_count {
            1 => {
                let prefix_budget =
                    remaining_budget / 2;

                let fuzzy_budget =
                    remaining_budget
                        - prefix_budget;

                let prefix_rest:
                    Vec<usize> =
                    prefix_indices
                        .iter()
                        .copied()
                        .skip(
                            TOP_PREFIX_COUNT
                        )
                        .take(
                            prefix_budget
                        )
                        .collect();

                let fuzzy =
                    self.fuzzy_results(
                        &query_lower,
                        fuzzy_budget,
                        &prefix_indices,
                    );

                self.filtered_apps =
                    top12
                        .into_iter()
                        .chain(
                            prefix_rest
                        )
                        .chain(fuzzy)
                        .take(
                            MAX_RESULTS
                        )
                        .collect();
            }

            2 => {
                let prefix_budget =
                    (remaining_budget * 3)
                        / 4;

                let fuzzy_budget =
                    remaining_budget
                        - prefix_budget;

                let prefix_rest:
                    Vec<usize> =
                    prefix_indices
                        .iter()
                        .copied()
                        .skip(
                            TOP_PREFIX_COUNT
                        )
                        .take(
                            prefix_budget
                        )
                        .collect();

                let fuzzy =
                    self.fuzzy_results(
                        &query_lower,
                        fuzzy_budget,
                        &prefix_indices,
                    );

                self.filtered_apps =
                    top12
                        .into_iter()
                        .chain(
                            prefix_rest
                        )
                        .chain(fuzzy)
                        .take(
                            MAX_RESULTS
                        )
                        .collect();
            }

            _ => {
                self.filtered_apps =
                    top12
                        .into_iter()
                        .chain(
                            prefix_indices
                                .into_iter()
                                .skip(
                                    TOP_PREFIX_COUNT
                                ),
                        )
                        .take(
                            MAX_RESULTS
                        )
                        .collect();
            }
        }
    }

    fn fuzzy_results(
        &self,
        query_lower: &str,
        budget: usize,
        exclude: &[usize],
    ) -> Vec<usize> {
        if budget == 0 {
            return Vec::new();
        }

        let pattern = Pattern::new(
            &self.query,
            CaseMatching::Smart,
            Normalization::Smart,
            AtomKind::Fuzzy,
        );

        let mut matcher =
            self.matcher.clone();

        let mut scored:
            Vec<(u32, usize)> = self
            .all_apps
            .iter()
            .enumerate()
            .filter(|(i, app)| {
                !exclude.contains(i)
                    && !app
                        .lower_name
                        .contains(
                            query_lower
                        )
            })
            .filter_map(|(i, app)| {
                pattern
                    .score(
                        app.haystack
                            .slice(..),
                        &mut matcher,
                    )
                    .map(|score| {
                        (score, i)
                    })
            })
            .collect();

        scored.sort_unstable_by(
            |a, b| {
                b.0.cmp(&a.0)
                    .then_with(|| {
                        self.all_apps[a.1]
                            .name
                            .cmp(
                                &self
                                    .all_apps
                                    [b.1]
                                    .name,
                            )
                    })
            },
        );

        scored
            .into_iter()
            .take(budget)
            .map(|(_, i)| i)
            .collect()
    }
}

// ── Persistence ───────────────────────────────────────────────────────────────

fn drawer_state_path() -> Option<PathBuf> {
    Some(
        dirs::data_local_dir()?
            .join("soulless")
            .join("drawers.json"),
    )
}

fn load_drawer_state() -> Option<DrawerState> {
    let path = drawer_state_path()?;

    let text = std::fs::read_to_string(path).ok()?;

    serde_json::from_str(&text).ok()
}

fn save_drawer_state(state: &DrawerState) {
    let Some(path) = drawer_state_path() else {
        return;
    };

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    match serde_json::to_string_pretty(state) {
        Ok(json) => {
            if let Err(e) =
                std::fs::write(&path, json)
            {
                eprintln!(
                    "DRAWER save failed: {}",
                    e
                );
            }
        }

        Err(e) => {
            eprintln!(
                "DRAWER serialize failed: {}",
                e
            );
        }
    }
}

// === DONE ===
// Fixed Vault::load() → Vault::new() :: done
// Fixed vault.setup() → vault.finish_setup() :: done
// Fixed vault.remove_entry() → vault.remove_file() :: done
// Removed unused vault_ui import :: done
// VaultOpenFile and VaultRemoveFile now handle errors into vault.error :: done
// VaultPasswordChanged and VaultConfirmChanged clear vault.error on input :: done
// All drawer management logic preserved unchanged :: done
// Search logic preserved unchanged :: done

// === DONE ===
// Replaced hardcoded drawers vec with DrawerState :: done
// Added app_by_id() lookup for drawer rendering :: done
// Added ContextMenu enum and app_picker state :: done
// Added full message set for drawer mutations :: done
// Added persistence: load on startup, save on every mutation :: done
// Search logic unchanged :: done
 
// === DONE ===
// Removed duplicate local AppEntry — now uses crate::indexer::AppEntry :: done
// Removed dead load_desktop_entries() — now uses crate::indexer::build_index() :: done
// Fixed empty query: alphabetical slice, not broken fuzzy-on-empty :: done
// Fixed tier logic: rank-based not raw storage index :: done
// Tiered spec:
//   0 chars  → first 200 alphabetical
//   1 char   → top 12 prefix, then 50% prefix / 50% fuzzy
//   2 chars  → top 12 prefix, then 75% prefix / 25% fuzzy
//   3+ chars → top 12 prefix, 100% prefix, 0% fuzzy :: done

// === DONE ===
// Rewritten search logic per MRV spec:
// 0 chars  → 200 results fuzzy (top 12 from that list fallback to prefix)
// 1 char   → top 12 prefix, then 25% prefix / 75% fuzzy
// 2 chars  → top 12 prefix, then 50% prefix / 50% fuzzy
// 3+ chars → top 12 prefix, then 100% prefix (no fuzzy)
// Removed icon-required filter — icon is now Option<String> with fallback
// Tightened should_skip_entry to exec-only
// No debug output in fuzzy path

// === DONE ===
// Added VaultFilesDropped(Vec<PathBuf>) message :: done
// Added VaultDragHover(bool) message :: done
// VaultDragHover sets vault.drag_hover for visual feedback :: done
// VaultFilesDropped iterates paths, calls vault.add_file() on each :: done
// NEEDS_PKEXEC errors counted as success — file is encrypted, original removal needs elevation :: done
// Clears drag_hover on drop regardless of outcome :: done
// Status shows count of added files; last hard error shown in red :: done
// All existing messages and logic unchanged :: done

// === DONE ===
// Fixed Vault::load() → Vault::new() :: done
// Fixed vault.setup() → vault.finish_setup() :: done
// Fixed vault.remove_entry() → vault.remove_file() :: done
// Removed unused vault_ui import :: done
// VaultOpenFile and VaultRemoveFile now handle errors into vault.error :: done
// VaultPasswordChanged and VaultConfirmChanged clear vault.error on input :: done
// All drawer management logic preserved unchanged :: done
// Search logic preserved unchanged :: done

// === DONE ===
// Replaced hardcoded drawers vec with DrawerState :: done
// Added app_by_id() lookup for drawer rendering :: done
// Added ContextMenu enum and app_picker state :: done
// Added full message set for drawer mutations :: done
// Added persistence: load on startup, save on every mutation :: done
// Search logic unchanged :: done
 
// === DONE ===
// Removed duplicate local AppEntry — now uses crate::indexer::AppEntry :: done
// Removed dead load_desktop_entries() — now uses crate::indexer::build_index() :: done
// Fixed empty query: alphabetical slice, not broken fuzzy-on-empty :: done
// Fixed tier logic: rank-based not raw storage index :: done
// Tiered spec:
//   0 chars  → first 200 alphabetical
//   1 char   → top 12 prefix, then 50% prefix / 50% fuzzy
//   2 chars  → top 12 prefix, then 75% prefix / 25% fuzzy
//   3+ chars → top 12 prefix, 100% prefix, 0% fuzzy :: done

// === DONE ===
// Rewritten search logic per MRV spec:
// 0 chars  → 200 results fuzzy (top 12 from that list fallback to prefix)
// 1 char   → top 12 prefix, then 25% prefix / 75% fuzzy
// 2 chars  → top 12 prefix, then 50% prefix / 50% fuzzy
// 3+ chars → top 12 prefix, then 100% prefix (no fuzzy)
// Removed icon-required filter — icon is now Option<String> with fallback
// Tightened should_skip_entry to exec-only
// No debug output in fuzzy path