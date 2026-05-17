// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.
 
use crate::indexer::{build_index, AppEntry};
use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};
 
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
        let all_apps = build_index();
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
 
        // Empty query: first 200 apps alphabetically.
        // all_apps is already sorted alphabetically by build_index().
        if self.query.is_empty() {
            self.filtered_apps = (0..self.all_apps.len().min(MAX_RESULTS)).collect();
            return;
        }
 
        let query_lower = self.query.to_lowercase();
        let char_count = self.query.chars().count();
 
        // Collect all prefix matches in alphabetical order (index order = alpha order)
        let prefix_indices: Vec<usize> = self
            .all_apps
            .iter()
            .enumerate()
            .filter(|(_, app)| app.lower_name.starts_with(&query_lower))
            .map(|(i, _)| i)
            .collect();
 
        // Top 12 prefix matches are always first in every tier
        let top12: Vec<usize> = prefix_indices.iter().copied().take(TOP_PREFIX_COUNT).collect();
        let remaining_budget = MAX_RESULTS.saturating_sub(top12.len());
 
        match char_count {
            // 1 char: after top 12 → 50% prefix, 50% fuzzy
            1 => {
                let prefix_budget = remaining_budget / 2;
                let fuzzy_budget = remaining_budget - prefix_budget;
 
                let prefix_rest: Vec<usize> = prefix_indices
                    .iter()
                    .copied()
                    .skip(TOP_PREFIX_COUNT)
                    .take(prefix_budget)
                    .collect();
 
                let fuzzy = self.fuzzy_results(&query_lower, fuzzy_budget, &prefix_indices);
 
                self.filtered_apps = top12
                    .into_iter()
                    .chain(prefix_rest)
                    .chain(fuzzy)
                    .take(MAX_RESULTS)
                    .collect();
            }
 
            // 2 chars: after top 12 → 75% prefix, 25% fuzzy
            2 => {
                let prefix_budget = (remaining_budget * 3) / 4;
                let fuzzy_budget = remaining_budget - prefix_budget;
 
                let prefix_rest: Vec<usize> = prefix_indices
                    .iter()
                    .copied()
                    .skip(TOP_PREFIX_COUNT)
                    .take(prefix_budget)
                    .collect();
 
                let fuzzy = self.fuzzy_results(&query_lower, fuzzy_budget, &prefix_indices);
 
                self.filtered_apps = top12
                    .into_iter()
                    .chain(prefix_rest)
                    .chain(fuzzy)
                    .take(MAX_RESULTS)
                    .collect();
            }
 
            // 3+ chars: 100% prefix, no fuzzy at all
            _ => {
                self.filtered_apps = top12
                    .into_iter()
                    .chain(prefix_indices.into_iter().skip(TOP_PREFIX_COUNT))
                    .take(MAX_RESULTS)
                    .collect();
            }
        }
    }
 
    /// Returns up to `budget` fuzzy-matched indices, skipping anything already in `exclude`.
    fn fuzzy_results(&self, query_lower: &str, budget: usize, exclude: &[usize]) -> Vec<usize> {
        if budget == 0 {
            return Vec::new();
        }
 
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
            .filter(|(i, app)| {
                !exclude.contains(i) && !app.lower_name.contains(query_lower)
            })
            .filter_map(|(i, app)| {
                pattern
                    .score(app.haystack.slice(..), &mut matcher)
                    .map(|score| (score, i))
            })
            .collect();
 
        scored.sort_unstable_by(|a, b| {
            b.0.cmp(&a.0)
                .then_with(|| self.all_apps[a.1].name.cmp(&self.all_apps[b.1].name))
        });
 
        scored.into_iter().take(budget).map(|(_, i)| i).collect()
    }
 
    pub fn drawers(&self) -> &[String] {
        &self.drawers
    }
}
 
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