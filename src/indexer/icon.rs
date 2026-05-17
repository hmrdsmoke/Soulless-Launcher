// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

pub const FALLBACK_ICON: &str = "assets/launcher.png";

/// Shared icon cache type.
///
/// Key:
///     Original icon name from .desktop file
///
/// Value:
///     Fully resolved absolute icon path
///
/// Example:
///     "firefox" -> "/usr/share/icons/hicolor/128x128/apps/firefox.png"
///
pub type SharedIconCache = Arc<RwLock<IconCache>>;

#[derive(Debug, Default)]
pub struct IconCache {
    cache: HashMap<String, String>,
}

impl IconCache {
    /// Creates a new empty icon cache.
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Resolve an icon name into an absolute path.
    ///
    /// Results are cached permanently after first lookup.
    ///
    /// This should ONLY happen during startup/app loading,
    /// never during rendering.
    pub fn resolve(
        &mut self,
        icon_name: Option<&str>,
    ) -> String {
        let Some(icon_name) = icon_name else {
            return FALLBACK_ICON.to_string();
        };

        // Cached hit
        if let Some(path) = self.cache.get(icon_name) {
            return path.clone();
        }

        // Resolve + cache
        let resolved = self
            .find_icon(icon_name)
            .unwrap_or_else(|| FALLBACK_ICON.to_string());

        self.cache
            .insert(icon_name.to_string(), resolved.clone());

        resolved
    }

    /// Performs the actual filesystem lookup.
    ///
    /// This is intentionally private.
    fn find_icon(
        &self,
        icon_name: &str,
    ) -> Option<String> {
        // Already absolute path
        if icon_name.starts_with('/') {
            if Path::new(icon_name).exists() {
                return Some(icon_name.to_string());
            }

            return None;
        }

        // Search order matters.
        //
        // Larger icons first.
        // SVG preferred when available.
        let search_dirs = [
            "/usr/share/icons/hicolor/scalable/apps",
            "/usr/share/icons/hicolor/256x256/apps",
            "/usr/share/icons/hicolor/128x128/apps",
            "/usr/share/icons/hicolor/96x96/apps",
            "/usr/share/icons/hicolor/64x64/apps",
            "/usr/share/icons/hicolor/48x48/apps",
            "/usr/share/pixmaps",
        ];

        let extensions = [
            "svg",
            "png",
            "xpm",
        ];

        for dir in &search_dirs {
            for ext in &extensions {
                let path = format!(
                    "{}/{}.{}",
                    dir,
                    icon_name,
                    ext
                );

                if Path::new(&path).exists() {
                    return Some(path);
                }
            }
        }

        None
    }

    /// Number of cached icons.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// True if cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Clears the cache.
    ///
    /// Mostly useful for development/testing.
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

/// Creates a shared thread-safe icon cache.
///
/// This is the preferred constructor for app-wide usage.
pub fn shared_cache() -> SharedIconCache {
    Arc::new(RwLock::new(IconCache::new()))
}

// === ARCHITECTURE NOTES ===
//
// GOAL:
//     Resolve ALL icons exactly once during startup.
//
// WHY:
//     Filesystem scanning during render is extremely expensive.
//
// CURRENT DESIGN:
//     search.rs loads desktop entries
//     -> icon.rs resolves icons
//     -> AppEntry stores resolved icon_path
//     -> drawers.rs only renders image(&app.icon_path)
//
// FUTURE IMPROVEMENTS:
//     - GTK/COSMIC theme integration
//     - symbolic icon variants
//     - image handle caching
//     - SVG raster cache
//     - async icon loading
//     - icon inheritance support
//
// PERFORMANCE TARGET:
//     Zero filesystem access during typing/rendering.
//
// === DONE ===
// Centralized icon resolution :: done
// One-time icon caching :: done
// Thread-safe shared cache :: done
// Fallback icon support :: done
// Freedesktop icon lookup skeleton :: done
// Rendering-time filesystem access eliminated :: done