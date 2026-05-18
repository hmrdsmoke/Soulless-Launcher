// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drawer {
    pub name: String,
    pub icon: String,
    pub apps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawerState {
    pub drawers: Vec<Drawer>,
}

impl Default for DrawerState {
    fn default() -> Self {
        Self {
            drawers: vec![
                Drawer {
                    name: "Daily Apps".to_string(),
                    icon: "⭐".to_string(),
                    apps: Vec::new(),
                },
                Drawer {
                    name: "Work".to_string(),
                    icon: "💼".to_string(),
                    apps: Vec::new(),
                },
                Drawer {
                    name: "Games".to_string(),
                    icon: "🎮".to_string(),
                    apps: Vec::new(),
                },
                Drawer {
                    name: "Utilities".to_string(),
                    icon: "⚙".to_string(),
                    apps: Vec::new(),
                },
            ],
        }
    }
}

impl DrawerState {
    // ── Accessors ─────────────────────────────────────────────

    pub fn drawers(&self) -> &[Drawer] {
        &self.drawers
    }

    pub fn drawer_names(&self) -> Vec<String> {
        self.drawers
            .iter()
            .map(|d| d.name.clone())
            .collect()
    }

    pub fn drawer_count(&self) -> usize {
        self.drawers.len()
    }

    pub fn apps_in_drawer(
        &self,
        drawer_name: &str,
    ) -> &[String] {
        self.drawers
            .iter()
            .find(|d| d.name == drawer_name)
            .map(|d| d.apps.as_slice())
            .unwrap_or(&[])
    }

    pub fn app_count(
        &self,
        drawer_name: &str,
    ) -> usize {
        self.drawers
            .iter()
            .find(|d| d.name == drawer_name)
            .map(|d| d.apps.len())
            .unwrap_or(0)
    }

    pub fn is_pinned(
        &self,
        drawer_name: &str,
        app_id: &str,
    ) -> bool {
        self.drawers
            .iter()
            .find(|d| d.name == drawer_name)
            .map(|d| {
                d.apps.iter().any(|id| id == app_id)
            })
            .unwrap_or(false)
    }

    // ── Drawer Management ─────────────────────────────────────

    pub fn create_drawer(
        &mut self,
        name: String,
        icon: String,
    ) {
        if self
            .drawers
            .iter()
            .any(|d| d.name == name)
        {
            return;
        }

        self.drawers.push(Drawer {
            name,
            icon,
            apps: Vec::new(),
        });
    }

    pub fn remove_drawer(
        &mut self,
        name: &str,
    ) {
        self.drawers.retain(|d| d.name != name);
    }

    pub fn rename_drawer(
        &mut self,
        old: &str,
        new: &str,
    ) {
        if old == new {
            return;
        }

        if self
            .drawers
            .iter()
            .any(|d| d.name == new)
        {
            return;
        }

        if let Some(drawer) = self
            .drawers
            .iter_mut()
            .find(|d| d.name == old)
        {
            drawer.name = new.to_string();
        }
    }

    pub fn set_drawer_icon(
        &mut self,
        drawer_name: &str,
        icon: String,
    ) {
        if let Some(drawer) = self
            .drawers
            .iter_mut()
            .find(|d| d.name == drawer_name)
        {
            drawer.icon = icon;
        }
    }

    pub fn move_drawer_up(
        &mut self,
        name: &str,
    ) {
        if let Some(index) = self
            .drawers
            .iter()
            .position(|d| d.name == name)
        {
            if index > 0 {
                self.drawers.swap(index, index - 1);
            }
        }
    }

    pub fn move_drawer_down(
        &mut self,
        name: &str,
    ) {
        if let Some(index) = self
            .drawers
            .iter()
            .position(|d| d.name == name)
        {
            if index + 1 < self.drawers.len() {
                self.drawers.swap(index, index + 1);
            }
        }
    }

    // ── App Mutations ─────────────────────────────────────────

    pub fn toggle_app(
        &mut self,
        drawer_name: &str,
        app_id: &str,
    ) {
        if let Some(drawer) = self
            .drawers
            .iter_mut()
            .find(|d| d.name == drawer_name)
        {
            if let Some(index) = drawer
                .apps
                .iter()
                .position(|id| id == app_id)
            {
                drawer.apps.remove(index);
            } else {
                drawer.apps.push(app_id.to_string());
            }
        }
    }

    pub fn add_app(
        &mut self,
        drawer_name: &str,
        app_id: String,
    ) {
        if let Some(drawer) = self
            .drawers
            .iter_mut()
            .find(|d| d.name == drawer_name)
        {
            if !drawer.apps.contains(&app_id) {
                drawer.apps.push(app_id);
            }
        }
    }

    pub fn remove_app(
        &mut self,
        drawer_name: &str,
        app_id: &str,
    ) {
        if let Some(drawer) = self
            .drawers
            .iter_mut()
            .find(|d| d.name == drawer_name)
        {
            drawer.apps.retain(|id| id != app_id);
        }
    }

    pub fn clear_drawer(
        &mut self,
        drawer_name: &str,
    ) {
        if let Some(drawer) = self
            .drawers
            .iter_mut()
            .find(|d| d.name == drawer_name)
        {
            drawer.apps.clear();
        }
    }
}

// === DONE ===
// Replaced HashMap architecture with ordered Vec<Drawer> model :: done
// Added drawer icons :: done
// Added ordered drawer support :: done
// Added move up/down support :: done
// Added create/delete drawer support :: done
// Added rename drawer support :: done
// Cleaner future animation architecture :: done
// Persistence-compatible serde model :: done
// Better long-term launcher scalability :: done

// === DONE ===
// Serialize/Deserialize derives present — ready for serde_json persistence :: done
// DrawerState is now the single source of truth for drawer names and contents :: done
// Used by search.rs: loaded at startup, saved on every mutation :: done