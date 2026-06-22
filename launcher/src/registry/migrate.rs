// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// src/registry/migrate.rs
// Migrates existing drawer app IDs to stable registry IDs.

use super::{Registry, EntryKind};
use crate::drawers::state::DrawerState;
use crate::search::indexer::AppEntry;

/// Walk all drawers and ensure every app ID is registered.
/// Replaces old source IDs with stable UUIDs in place.
pub fn migrate_drawers(
    drawer_state: &mut DrawerState,
    registry: &mut Registry,
    all_apps: &[AppEntry],
) -> bool {
    let mut changed = false;
    for drawer in drawer_state.drawers.iter_mut() {
        let mut new_apps = Vec::new();
        for app_id in &drawer.apps {
            // Already a UUID — already migrated
            if uuid::Uuid::parse_str(app_id).is_ok() {
                new_apps.push(app_id.clone());
                continue;
            }
            // Find app in index by source ID
            if let Some(app) = all_apps.iter().find(|a| &a.id == app_id) {
                let stable_id = registry.register_app(
                    &app.id,
                    &app.name,
                    &app.exec,
                    &app.icon_path,
                );
                new_apps.push(stable_id);
                changed = true;
            } else {
                // App not found — keep original ID for now
                new_apps.push(app_id.clone());
            }
        }
        drawer.apps = new_apps;

        // Migrate files to registry
        for file in &drawer.files {
            let _ = registry.register_path(&file.path, EntryKind::File);
        }
    }
    changed
}

/// Remove drawer app entries that no longer resolve to any installed app.
/// Returns true if anything was removed.
pub fn prune_dead_apps(
    drawer_state: &mut DrawerState,
    registry: &Registry,
    all_apps: &[AppEntry],
) -> bool {
    let mut changed = false;
    for drawer in drawer_state.drawers.iter_mut() {
        let before = drawer.apps.len();
        drawer.apps.retain(|app_id| {
            crate::search::indexer::appid::resolve(app_id, all_apps, registry).is_some()
        });
        if drawer.apps.len() != before {
            changed = true;
        }
    }
    changed
}
