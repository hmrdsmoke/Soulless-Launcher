use super::AppEntry;
use std::fs;
use std::path::PathBuf;

const CACHE_DIR: &str = ".cache/soulless";
const CACHE_FILE: &str = "index.bin";

pub fn cache_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;

    Some(
        home.join(CACHE_DIR)
            .join(CACHE_FILE),
    )
}

pub fn load_cache() -> Option<Vec<AppEntry>> {
    let path = cache_path()?;

    // Cache not created yet
    if !path.exists() {
        return None;
    }

    // Stub for now
    //
    // Later:
    // - deserialize binary cache
    // - restore launch stats
    // - restore metadata
    //
    // For now we just prove the architecture works.

    eprintln!(
        "CACHE found at {}",
        path.display()
    );

    None
}

pub fn save_cache(apps: &[AppEntry]) {
    let Some(path) = cache_path() else {
        return;
    };

    // Ensure cache directory exists
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    // Stub write for now
    //
    // Later:
    // - bincode
    // - postcard
    // - rkyv
    // - compressed binary cache
    //
    // For now:
    // just create placeholder file.

    let contents = format!(
        "soulless cache placeholder\napps={}",
        apps.len()
    );

    match fs::write(&path, contents) {
        Ok(_) => {
            eprintln!(
                "CACHE saved {} apps -> {}",
                apps.len(),
                path.display()
            );
        }

        Err(err) => {
            eprintln!(
                "CACHE save failed: {}",
                err
            );
        }
    }
}