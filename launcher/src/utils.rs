// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// src/utils.rs
// Shared utility functions used across the launcher.

/// Strip desktop entry exec field placeholders like %f, %u, %F, %U etc.
pub fn strip_desktop_placeholders(exec: &str) -> String {
    let mut result = String::with_capacity(exec.len());
    let mut chars = exec.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            if chars
                .peek()
                .map_or(false, |&next| next.is_ascii_alphabetic())
            {
                chars.next();
                continue;
            }
        }
        result.push(c);
    }

    result.trim().to_string()
}

/// Decode a percent-encoded URI path (e.g. file:// drag and drop payloads).
pub fn percent_decode_uri(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (
                hex_nibble(bytes[i + 1]),
                hex_nibble(bytes[i + 2]),
            ) {
                out.push((hi << 4 | lo) as char);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }

    out
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
