// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawerState {
    pub drawers: HashMap<String, Vec<String>>,
}

impl Default for DrawerState {
    fn default() -> Self {
        let mut drawers = HashMap::new();

        drawers.insert(
            "Daily Apps".to_string(),
            Vec::new(),
        );

        drawers.insert(
            "Work".to_string(),
            Vec::new(),
        );

        drawers.insert(
            "Games".to_string(),
            Vec::new(),
        );

        drawers.insert(
            "Utilities".to_string(),
            Vec::new(),
        );

        Self { drawers }
    }
}

impl DrawerState {
    pub fn add_drawer(&mut self, name: &str) {
        self.drawers
            .entry(name.to_string())
            .or_default();
    }

    pub fn remove_drawer(&mut self, name: &str) {
        self.drawers.remove(name);
    }

    pub fn rename_drawer(
        &mut self,
        old: &str,
        new: &str,
    ) {
        if old == new {
            return;
        }

        if let Some(apps) =
            self.drawers.remove(old)
        {
            self.drawers
                .insert(new.to_string(), apps);
        }
    }

    pub fn toggle_app(
        &mut self,
        drawer: &str,
        app_id: &str,
    ) {
        let apps = self
            .drawers
            .entry(drawer.to_string())
            .or_default();

        if let Some(index) =
            apps.iter().position(|id| id == app_id)
        {
            apps.remove(index);
        } else {
            apps.push(app_id.to_string());
        }
    }

    pub fn add_app(
        &mut self,
        drawer: &str,
        app_id: String,
    ) {
        let apps = self
            .drawers
            .entry(drawer.to_string())
            .or_default();

        if !apps.contains(&app_id) {
            apps.push(app_id);
        }
    }

    pub fn remove_app(
        &mut self,
        drawer: &str,
        app_id: &str,
    ) {
        if let Some(apps) =
            self.drawers.get_mut(drawer)
        {
            apps.retain(|id| id != app_id);
        }
    }

    pub fn is_pinned(
        &self,
        drawer: &str,
        app_id: &str,
    ) -> bool {
        self.drawers
            .get(drawer)
            .map(|apps| {
                apps.iter().any(|id| id == app_id)
            })
            .unwrap_or(false)
    }

    pub fn apps_in_drawer(
        &self,
        drawer: &str,
    ) -> &[String] {
        self.drawers
            .get(drawer)
            .map(|apps| apps.as_slice())
            .unwrap_or(&[])
    }

    pub fn drawer_names(&self) -> Vec<String> {
        let mut names: Vec<String> =
            self.drawers.keys().cloned().collect();

        names.sort();

        names
    }

    pub fn drawer_count(&self) -> usize {
        self.drawers.len()
    }

    pub fn app_count(
        &self,
        drawer: &str,
    ) -> usize {
        self.drawers
            .get(drawer)
            .map(|apps| apps.len())
            .unwrap_or(0)
    }

    pub fn clear_drawer(
        &mut self,
        drawer: &str,
    ) {
        if let Some(apps) =
            self.drawers.get_mut(drawer)
        {
            apps.clear();
        }
    }
}

// === DONE ===
// Serialize/Deserialize derives present — ready for serde_json persistence :: done
// DrawerState is now the single source of truth for drawer names and contents :: done
// Used by search.rs: loaded at startup, saved on every mutation :: done