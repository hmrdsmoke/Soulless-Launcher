// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// src/registry/mod.rs
// Stable app/file/dir ID registry.
// Maps stable UUIDs to app metadata so drawer entries survive renames and source changes.

pub mod migrate;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntryKind {
    App,
    File,
    Dir,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    /// Stable UUID — never changes
    pub id: String,
    pub kind: EntryKind,
    pub name: String,
    /// exec string for App, absolute path for File/Dir
    pub exec_or_path: String,
    pub icon: String,
    /// Original source ID e.g. "desktop:Firefox.desktop"
    pub source_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Registry {
    /// stable_id -> entry
    pub entries: HashMap<String, RegistryEntry>,
    /// source_id -> stable_id (for fast lookup during indexing)
    pub source_index: HashMap<String, String>,
}

impl Registry {
    pub fn get(&self, stable_id: &str) -> Option<&RegistryEntry> {
        self.entries.get(stable_id)
    }

    /// Register an app from the indexer. Returns stable ID.
    pub fn register_app(
        &mut self,
        source_id: &str,
        name: &str,
        exec: &str,
        icon: &str,
    ) -> String {
        if let Some(id) = self.source_index.get(source_id) {
            return id.clone();
        }
        let id = uuid::Uuid::new_v4().to_string();
        let entry = RegistryEntry {
            id: id.clone(),
            kind: EntryKind::App,
            name: name.to_string(),
            exec_or_path: exec.to_string(),
            icon: icon.to_string(),
            source_id: source_id.to_string(),
        };
        self.source_index.insert(source_id.to_string(), id.clone());
        self.entries.insert(id.clone(), entry);
        id
    }

    /// Register a file or directory. Returns stable ID.
    pub fn register_path(&mut self, path: &str, kind: EntryKind) -> String {
        let key = format!("path:{}", path);
        if let Some(id) = self.source_index.get(&key) {
            return id.clone();
        }
        let id = uuid::Uuid::new_v4().to_string();
        let name = PathBuf::from(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path)
            .to_string();
        let entry = RegistryEntry {
            id: id.clone(),
            kind,
            name,
            exec_or_path: path.to_string(),
            icon: String::new(),
            source_id: key.clone(),
        };
        self.source_index.insert(key, id.clone());
        self.entries.insert(id.clone(), entry);
        id
    }
}

fn registry_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("soulless")
        .join("app_registry.json")
}

pub fn load() -> Registry {
    let path = registry_path();
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Registry::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

pub fn save(registry: &Registry) {
    let path = registry_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(registry) {
        let _ = std::fs::write(&path, json);
    }
}
