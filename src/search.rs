// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

use freedesktop_desktop_entry::DesktopEntry;
use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32String};
use std::fs;

#[derive(Clone)]
pub enum Message {
    QueryChanged(String),
    AppClicked(String),
    DrawerClicked(String),
    VaultClicked,
    SearchBarClicked,
}

#[derive(Clone, PartialEq, Eq)]
pub enum OpenDrawer {
    Search,
    Pinned(String),
    Vault,
}

#[derive(Clone)]
pub struct AppEntry {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>, // None = use fallback asset
    lower_name: String,
    haystack: Utf32String,
}

pub struct Search {
    pub query: String,
    matcher: Matcher,
    all_apps: Vec<AppEntry>,
    filtered_apps: Vec<usize>,
    drawers: Vec<String>,
    pub show_search_results: bool,
    pub current_open_drawer: OpenDrawer,
}

impl Search {
    pub fn new() -> Self {
        let matcher = Matcher::new(Config::DEFAULT);
        let all_apps = load_desktop_entries();
        let drawers = vec![
            "Utilities".to_string(),
            "Daily Apps".to_string(),
            "Work".to_string(),
            "Games".to_string(),
        ];

        let mut search = Self {
            query: String::new(),
            matcher,
            all_apps,
            filtered_apps: Vec::new(),
            drawers,
            show_search_results: true,
            current_open_drawer: OpenDrawer::Search,
        };

        search.recompute_results();
        search
    }

    pub fn update(&mut self, message: Message) -> Option<String> {
        match message {
            Message::QueryChanged(q) => {
                self.query = q;
                self.recompute_results();
                self.show_search_results = true;
                self.current_open_drawer = OpenDrawer::Search;
                None
            }
            Message::AppClicked(exec) => Some(exec),
            Message::DrawerClicked(name) => {
                self.current_open_drawer = OpenDrawer::Pinned(name);
                self.show_search_results = false;
                None
            }
            Message::VaultClicked => {
                self.current_open_drawer = OpenDrawer::Vault;
                self.show_search_results = false;
                None
            }
            Message::SearchBarClicked => {
                self.show_search_results = true;
                self.current_open_drawer = OpenDrawer::Search;
                None
            }
        }
    }

    pub fn filtered_apps(&self) -> &[usize] {
        &self.filtered_apps
    }

    pub fn app(&self, index: usize) -> Option<&AppEntry> {
        self.all_apps.get(index)
    }

    fn recompute_results(&mut self) {
        const MAX_RESULTS: usize = 200;
        const TOP_PREFIX_COUNT: usize = 12;

        if self.query.is_empty() {
            // No input: top 200 results based on 100% fuzzy score
            let pattern = Pattern::new(
                &self.query,
                CaseMatching::Smart,
                Normalization::Smart,
                AtomKind::Fuzzy,
            );
            let mut matcher = self.matcher.clone();

            let mut scored: Vec<(u32, usize)> = self
                .all_apps
                .iter()
                .enumerate()
                .filter_map(|(i, app)| {
                    pattern
                        .score(app.haystack.slice(..), &mut matcher)
                        .map(|fuzzy_score| (fuzzy_score, i))
                })
                .collect();

            // Sort by score (descending), then alphabetically
            scored.sort_unstable_by(|a, b| {
                b.0.cmp(&a.0)
                    .then_with(|| self.all_apps[a.1].name.cmp(&self.all_apps[b.1].name))
            });

            // Take top 200 results
            self.filtered_apps = scored
                .into_iter()
                .take(MAX_RESULTS)
                .map(|(_, i)| i)
                .collect();
            return;
        }

        let query_lower = self.query.to_lowercase();
        let char_count = self.query.chars().count();

        // Create a fuzzy pattern matcher
        let pattern = Pattern::new(
            &self.query,
            CaseMatching::Smart,
            Normalization::Smart,
            AtomKind::Fuzzy,
        );
        let mut matcher = self.matcher.clone();

        // Compute scores for all apps
        let mut scored: Vec<(u32, usize)> = self
            .all_apps
            .iter()
            .enumerate()
            .filter_map(|(i, app)| {
                let mut score = 0;

                // Prefix match: base score
                if app.lower_name.starts_with(&query_lower) {
                    score += 20_000; // Strong prefix bonus
                }

                let fuzzy_score = pattern.score(app.haystack.slice(..), &mut matcher).unwrap_or(0);

                let final_score = match char_count {
                    1 => {
                        if i < TOP_PREFIX_COUNT {
                            score // Top 12: pure prefix score
                        } else {
                            (score as f32 * 0.25 + fuzzy_score as f32 * 0.75) as u32 // After top 12: 25% prefix, 75% fuzzy
                        }
                    }
                    2 => {
                        if i < TOP_PREFIX_COUNT {
                            score // Top 12: pure prefix score
                        } else {
                            (score as f32 * 0.50 + fuzzy_score as f32 * 0.50) as u32 // After top 12: 50/50
                        }
                    }
                    _ => {
                        // 3 or more characters
                        if i < TOP_PREFIX_COUNT {
                            score // Top 12: pure prefix score
                        } else {
                            score // After top 12: still pure prefix
                        }
                    }
                };

                Some((final_score, i))
            })
            .collect();

        // Sort by final score descending, then alphabetically
        scored.sort_unstable_by(|a, b| {
            b.0.cmp(&a.0)
                .then_with(|| self.all_apps[a.1].name.cmp(&self.all_apps[b.1].name))
        });

        // Take top 200 results
        self.filtered_apps = scored
            .into_iter()
            .take(MAX_RESULTS)
            .map(|(_, i)| i)
            .collect();
    }

    pub fn drawers(&self) -> &[String] {
        &self.drawers
    }
}

fn load_desktop_entries() -> Vec<AppEntry> {
    let mut apps = Vec::new();
    let home = dirs::home_dir().unwrap_or_default();

    let dirs = [
        "/usr/share/applications".to_string(),
        "/usr/local/share/applications".to_string(),
        format!("{}/.local/share/applications", home.display()),
        "/var/lib/flatpak/exports/share/applications".to_string(),
        format!(
            "{}/.local/share/flatpak/exports/share/applications",
            home.display()
        ),
    ];

    for dir in dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                    if let Ok(desktop) = DesktopEntry::from_path::<&str>(path, &[]) {
                        if let Some(name) = desktop.name::<&str>(&[]) {
                            if let Some(exec) = desktop.exec() {
                                if should_skip_entry(exec) {
                                    continue;
                                }

                                let clean_exec = crate::strip_desktop_placeholders(exec);
                                let name_str = name.to_string();
                                let icon = desktop.icon().map(|i| i.to_string());

                                apps.push(AppEntry {
                                    lower_name: name_str.to_lowercase(),
                                    haystack: Utf32String::from(name_str.as_str()),
                                    name: name_str,
                                    exec: clean_exec,
                                    icon,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    apps.sort_by(|a, b| a.name.cmp(&b.name));
    apps
}

fn should_skip_entry(exec: &str) -> bool {
    // Only filter on exec — never on name, to avoid dropping valid user apps
    let lower_exec = exec.to_lowercase();

    let suspicious_exec_terms = [
        "handler", "oauth", "daemon", "service", "portal",
    ];

    suspicious_exec_terms
        .iter()
        .any(|term| lower_exec.contains(term))
}

// === DONE ===
// Rewritten search logic per MRV spec:
// 0 chars  → 200 results fuzzy (top 12 from that list fallback to prefix)
// 1 char   → top 12 prefix, then 25% prefix / 75% fuzzy
// 2 chars  → top 12 prefix, then 50% prefix / 50% fuzzy
// 3+ chars → top 12 prefix, then 100% prefix (no fuzzy)
// Removed icon-required filter — icon is now Option<String> with fallback
// Tightened should_skip_entry to exec-only
// No debug output in fuzzy path