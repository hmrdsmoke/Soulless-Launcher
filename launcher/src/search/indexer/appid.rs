// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// src/search/indexer/appid.rs
// Resolves stable registry IDs to AppEntry.
// Keeps registry concerns out of Search.

use super::AppEntry;
use crate::registry::Registry;

/// Resolve a stable UUID or legacy source ID to an AppEntry.
pub fn resolve<'a>(id: &str, all_apps: &'a [AppEntry], registry: &Registry) -> Option<&'a AppEntry> {
    // Direct match — legacy source ID still in use
    if let Some(app) = all_apps.iter().find(|a| a.id == id) {
        return Some(app);
    }
    // Registry lookup — stable UUID
    if let Some(entry) = registry.get(id) {
        return all_apps.iter().find(|a| a.id == entry.source_id);
    }
    None
}
