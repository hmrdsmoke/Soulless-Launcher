// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

pub const FALLBACK_ICON: &str = "assets/launcher.png";

pub fn fallback_icon() -> String {
    "/usr/share/icons/hicolor/48x48/apps/apport.png".to_string()
}

#[allow(dead_code)] // issue #6 — icon cache utilities
/// Shared icon cache type.
pub type SharedIconCache = Arc<RwLock<IconCache>>;

#[derive(Debug, Default)]
pub struct IconCache {
    /// icon_name → resolved PNG/JPEG path (never SVG)
    cache: HashMap<String, String>,
    /// Directory where rasterized SVGs are written
    svg_cache_dir: PathBuf,
}

impl IconCache {
    pub fn new() -> Self {
        let svg_cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("soulless")
            .join("icons");

        let _ = std::fs::create_dir_all(&svg_cache_dir);

        Self {
            cache: HashMap::new(),
            svg_cache_dir,
        }
    }

    /// Resolve an icon name to an absolute raster (PNG/JPEG) path.
    /// SVGs are rasterized to PNG on first use and cached to disk.
    /// Results are kept in memory after first lookup.
    pub fn resolve(&mut self, icon_name: Option<&str>) -> String {
        let Some(icon_name) = icon_name else {
            return fallback_icon();
        };

        if let Some(path) = self.cache.get(icon_name) {
            return path.clone();
        }

        let resolved = self
            .find_icon(icon_name)
            .or_else(|| {
                if icon_name.ends_with("-symbolic") {
                    let base = &icon_name[..icon_name.len() - 9];
                    self.find_icon(base)
                } else {
                    self.find_icon(&format!("{}-symbolic", icon_name))
                }
            })
            .unwrap_or_else(|| {
                eprintln!("ICON MISS: {}", icon_name);
                fallback_icon()
            });

        self.cache.insert(icon_name.to_string(), resolved.clone());
        resolved
    }

    fn find_icon(&self, icon_name: &str) -> Option<String> {
        // steam_icon_<appid> — Steam generates these for installed game .desktop files.
        // Map them directly to the librarycache header.jpg.
        if let Some(appid) = icon_name.strip_prefix("steam_icon_") {
            let home = dirs::home_dir().unwrap_or_default();
            let cache_dir = home
                .join(".local/share/Steam/appcache/librarycache")
                .join(appid);
            let header = cache_dir.join("header.jpg");
            if header.exists() {
                return Some(header.display().to_string());
            }
            // Fall back to first jpg in folder
            if let Ok(entries) = std::fs::read_dir(&cache_dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.extension().and_then(|e| e.to_str()) == Some("jpg") {
                        return Some(p.display().to_string());
                    }
                }
            }
            return None;
        }

        // Already an absolute path
        if icon_name.starts_with('/') {
            if Path::new(icon_name).exists() {
                // If it's an SVG, rasterize it
                if icon_name.ends_with(".svg") {
                    return self.rasterize_svg(icon_name);
                }
                return Some(icon_name.to_string());
            }
            return None;
        }

        // Search order: large PNGs first (best quality for display),
        // then smaller PNGs, then SVGs (which get rasterized).
        // cosmic::iced::widget::image() is raster-only — SVGs must be
        // converted to PNG before use.
        // Icon categories beyond "apps" — needed for generic freedesktop names
        // like dialog-information (status), drive-removable-media (devices),
        // document-new (actions), bluetooth (status), input-keyboard (devices)
        let categories = ["apps", "actions", "devices", "status", "mimetypes", "places", "categories"];
        let sizes = ["256x256", "128x128", "96x96", "64x64", "48x48", "32x32"];
        let home = dirs::home_dir().unwrap_or_default();
        let local_hicolor = home.join(".local/share/icons/hicolor");

        // Build PNG search dirs across all sizes and categories
        let mut png_dirs: Vec<String> = Vec::new();
        for size in &sizes {
            for cat in &categories {
                // User-local icons first (Chrome app shortcuts etc)
                png_dirs.push(format!("{}/{}/{}", local_hicolor.display(), size, cat));
                png_dirs.push(format!("/usr/share/icons/hicolor/{}/{}", size, cat));
                png_dirs.push(format!("/usr/share/icons/Pop/{}/{}", size, cat));
            }
        }
        png_dirs.push("/usr/share/pixmaps".to_string());

        // Build SVG search dirs across all categories
        let mut svg_dirs: Vec<String> = Vec::new();
        for cat in &categories {
            svg_dirs.push(format!("{}/scalable/{}", local_hicolor.display(), cat));
            svg_dirs.push(format!("/usr/share/icons/hicolor/scalable/{}", cat));
            svg_dirs.push(format!("/usr/share/icons/Cosmic/scalable/{}", cat));
            svg_dirs.push(format!("/usr/share/icons/Pop/scalable/{}", cat));
        }
        // Breeze theme — has preferences/ category icons
        svg_dirs.push("/usr/share/icons/breeze/preferences/32".to_string());
        svg_dirs.push("/usr/share/icons/breeze/preferences/22".to_string());

        // Try raster formats first (no conversion needed), then SVG in same dirs
        for dir in png_dirs.iter() {
            // PNG
            let path = format!("{}/{}.png", dir, icon_name);
            if Path::new(&path).exists() {
                return Some(path);
            }
            // JPEG
            let path = format!("{}/{}.jpg", dir, icon_name);
            if Path::new(&path).exists() {
                return Some(path);
            }
            // SVG inside a sized dir (e.g. hicolor/256x256/apps/com.system76.CosmicFiles.svg)
            let path = format!("{}/{}.svg", dir, icon_name);
            if Path::new(&path).exists() {
                return self.rasterize_svg(&path);
            }
        }

        // Try SVG — rasterize to PNG
        for dir in svg_dirs.iter() {
            let path = format!("{}/{}.svg", dir, icon_name);
            if Path::new(&path).exists() {
                return self.rasterize_svg(&path);
            }
        }

        // Try basename only (handles icon names with path components)
        if icon_name.contains('/') {
            let basename = icon_name.rsplit('/').next().unwrap_or(icon_name);
            if basename != icon_name {
                return self.find_icon(basename);
            }
        }

        None
    }

    /// Rasterize an SVG to a PNG in the svg_cache_dir.
    /// Returns the PNG path, or None if rasterization fails.
    fn rasterize_svg(&self, svg_path: &str) -> Option<String> {
        use resvg::usvg::{Options, Tree};
        use resvg::tiny_skia::{Pixmap, Transform};

        // Use a stable hash of the full path to avoid filename collisions
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        svg_path.hash(&mut hasher);
        let cache_key = hasher.finish();
        let out_path = self.svg_cache_dir.join(format!("{:016x}.png", cache_key));

        // Already rasterized
        if out_path.exists() {
            return Some(out_path.display().to_string());
        }

        let svg_data = std::fs::read(svg_path).ok()?;

        let opts = Options::default();
        let tree = Tree::from_data(&svg_data, &opts).ok()?;

        let size = tree.size();
        let width = (size.width() as u32).max(1).min(256);
        let height = (size.height() as u32).max(1).min(256);

        // Scale to 64x64 for consistent display
        let target = 64u32;
        let scale_x = target as f32 / width as f32;
        let scale_y = target as f32 / height as f32;
        let scale = scale_x.min(scale_y);

        let px_w = ((width as f32 * scale) as u32).max(1);
        let px_h = ((height as f32 * scale) as u32).max(1);

        let mut pixmap = Pixmap::new(px_w, px_h)?;

        resvg::render(
            &tree,
            Transform::from_scale(scale, scale),
            &mut pixmap.as_mut(),
        );

        pixmap.save_png(&out_path).ok()?;

        Some(out_path.display().to_string())
    }

    pub fn len(&self) -> usize { self.cache.len() }
    pub fn is_empty(&self) -> bool { self.cache.is_empty() }
    pub fn clear(&mut self) { self.cache.clear(); }
}

pub fn shared_cache() -> SharedIconCache {
    Arc::new(RwLock::new(IconCache::new()))
}