// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/utils.rs
// Shared utility functions used across the launcher.

/// Strip desktop entry exec field placeholders like %f, %u, %F, %U etc.
pub fn strip_desktop_placeholders(exec: &str) -> String {
    let mut result = String::with_capacity(exec.len());
    let mut chars = exec.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%'
            && chars
                .peek()
                .is_some_and(|&next| next.is_ascii_alphabetic())
            {
                chars.next();
                continue;
            }
        result.push(c);
    }

    result.trim().to_string()
}

/// Decode a percent-encoded URI path (e.g. file:// drag and drop payloads).
///
/// Decodes into a BYTE buffer, not a String. A percent-encoded non-ASCII
/// character is multiple bytes (é is %C3%A9); pushing each decoded byte as a
/// `char` treats it as a Latin-1 codepoint and yields mojibake, so the path
/// never matches a real file and the drop is silently discarded. Reassembling
/// at the end lets UTF-8 sequences come back out whole.
pub fn percent_decode_uri(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len()
            && let (Some(hi), Some(lo)) = (
                hex_nibble(bytes[i + 1]),
                hex_nibble(bytes[i + 2]),
            ) {
                out.push(hi << 4 | lo);
                i += 3;
                continue;
            }
        out.push(bytes[i]);
        i += 1;
    }

    String::from_utf8_lossy(&out).into_owned()
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Truncate a label to max characters, appending ellipsis if needed.
pub fn truncate_label(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_string()
    } else {
        let mut s: String = text.chars().take(max.saturating_sub(1)).collect();
        s.push('…');
        s
    }
}

/// Parse a `text/uri-list` DnD payload into existing local file paths.
/// Decodes percent-encoding and keeps only entries that are existing `file://` paths.
pub fn parse_uri_list(data: &[u8]) -> Vec<std::path::PathBuf> {
    let payload = String::from_utf8_lossy(data);
    payload
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("file://"))
        .filter_map(|l| {
            let raw = l.trim_start_matches("file://");
            let decoded = percent_decode_uri(raw);
            let p = std::path::PathBuf::from(decoded);
            if p.exists() { Some(p) } else { None }
        })
        .collect()
}

/// Launch a command string the way a launcher must: on the HOST.
///
/// Natively that is just `sh -c <exec>`. Inside Flatpak the sandbox contains
/// none of the host's binaries, so `sh -c firefox` finds nothing and the click
/// does nothing. `flatpak-spawn --host` is Flatpak's portal-mediated escape
/// hatch for exactly this (requires --talk-name=org.freedesktop.Flatpak in
/// finish-args). Without it the launcher can index 1734 apps and launch zero.
///
/// Args are passed as separate argv entries, so no shell re-quoting happens.
/// The child is reaped on a detached thread — the launcher is a long-lived
/// daemon and must not accumulate zombies.
/// Minimal shell escaping: wraps a string in single quotes and escapes any
/// embedded single quotes. Inside single quotes the shell treats `, $(), ;, |,
/// and whitespace as literal — so a path wrapped this way can never inject when
/// it reaches `sh -c`. Use on any filesystem path spliced into an exec string.
pub fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

pub fn spawn_exec(exec: &str) {
    let clean = strip_desktop_placeholders(exec);

    let mut cmd = if crate::search::indexer::hostpath::sandboxed() {
        let mut c = std::process::Command::new("flatpak-spawn");
        c.args(["--host", "sh", "-c"]).arg(&clean);
        c
    } else {
        let mut c = std::process::Command::new("sh");
        c.arg("-c").arg(&clean);
        c
    };

    match cmd.spawn() {
        Ok(mut child) => {
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        }
        Err(e) => eprintln!("[launch] failed to spawn `{clean}`: {e}"),
    }
}
