// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/search/indexer/hostpath.rs
// Host path resolution for sandboxed runs.
//
// Flatpak refuses --filesystem=/usr (reserved). The sanctioned route is
// --filesystem=host-os:ro, which mounts the host's /usr at /run/host/usr.
// Inside the sandbox the host's apps, icons, and binaries therefore live one
// prefix deeper. Native runs get an empty prefix and behave exactly as before.

/// "/run/host" when running inside Flatpak, "" natively.
pub fn host_prefix() -> &'static str {
    static PREFIX: std::sync::OnceLock<&'static str> = std::sync::OnceLock::new();
    PREFIX.get_or_init(|| {
        if std::env::var_os("FLATPAK_ID").is_some() && std::path::Path::new("/run/host/usr").exists()
        {
            "/run/host"
        } else {
            ""
        }
    })
}

/// Rewrite an absolute host path for the current environment.
/// `/usr/share/icons` -> `/run/host/usr/share/icons` when sandboxed.
pub fn host(path: &str) -> String {
    format!("{}{}", host_prefix(), path)
}

/// True when running inside the Flatpak sandbox.
pub fn sandboxed() -> bool {
    !host_prefix().is_empty()
}