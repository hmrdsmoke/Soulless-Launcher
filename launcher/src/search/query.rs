// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// src/search/query.rs
// Smart query interpretation for the Soulless launcher.
// Parses natural language queries and returns filtered app indices.
// Add new query types here — search/mod.rs just calls interpret().

use crate::search::indexer::{AppEntry, AppSource};

const DAY: u64 = 86400;

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Parse a number word or digit from a query string.
/// "last 5" → 5, "last ten" → 10, "last" → default
fn parse_count(query: &str, default: usize) -> usize {
    let word_numbers = [
        ("one", 1), ("two", 2), ("three", 3), ("four", 4), ("five", 5),
        ("six", 6), ("seven", 7), ("eight", 8), ("nine", 9), ("ten", 10),
        ("fifteen", 15), ("twenty", 20), ("thirty", 30), ("fifty", 50),
        ("hundred", 100),
    ];
    for word in query.split_whitespace() {
        if let Ok(n) = word.parse::<usize>() {
            return n;
        }
        for (w, n) in &word_numbers {
            if word == *w {
                return *n;
            }
        }
    }
    default
}

/// Returns Some(filtered indices) if the query is a smart query, None otherwise.
pub fn interpret(query: &str, apps: &[AppEntry]) -> Option<Vec<usize>> {
    let q = query.trim().to_lowercase();
    let now = now_secs();

    // ── Source filters ────────────────────────────────────────────────────────
    if q == "files" || q == "file" {
        return Some(apps.iter().enumerate()
            .filter(|(_, a)| matches!(a.source, AppSource::File))
            .map(|(i, _)| i).collect());
    }
    if q == "apps" || q == "app" {
        return Some(apps.iter().enumerate()
            .filter(|(_, a)| matches!(a.source, AppSource::Desktop | AppSource::Flatpak | AppSource::AppImage))
            .map(|(i, _)| i).collect());
    }
    if q == "games" || q == "game" {
        return Some(apps.iter().enumerate()
            .filter(|(_, a)| matches!(a.source, AppSource::Steam))
            .map(|(i, _)| i).collect());
    }
    if q == "flatpak" || q == "flatpaks" {
        return Some(apps.iter().enumerate()
            .filter(|(_, a)| matches!(a.source, AppSource::Flatpak))
            .map(|(i, _)| i).collect());
    }
    if q == "appimage" || q == "appimages" {
        return Some(apps.iter().enumerate()
            .filter(|(_, a)| matches!(a.source, AppSource::AppImage))
            .map(|(i, _)| i).collect());
    }
    if q == "cli" || q == "terminal" || q == "commands" {
        return Some(apps.iter().enumerate()
            .filter(|(_, a)| matches!(a.source, AppSource::Binary))
            .map(|(i, _)| i).collect());
    }

    // ── Time-based queries ────────────────────────────────────────────────────
    if q == "today" {
        return Some(apps.iter().enumerate()
            .filter(|(_, a)| a.last_launched.is_some_and(|t| now - t < DAY))
            .map(|(i, _)| i).collect());
    }
    if q == "yesterday" {
        return Some(apps.iter().enumerate()
            .filter(|(_, a)| a.last_launched.is_some_and(|t| {
                let age = now - t; (DAY..DAY * 2).contains(&age)
            }))
            .map(|(i, _)| i).collect());
    }
    if q == "this week" || q == "week" {
        return Some(apps.iter().enumerate()
            .filter(|(_, a)| a.last_launched.is_some_and(|t| now - t < DAY * 7))
            .map(|(i, _)| i).collect());
    }

    // ── Usage-based queries ───────────────────────────────────────────────────
    if q == "never used" || q == "unused" {
        return Some(apps.iter().enumerate()
            .filter(|(_, a)| a.launch_count == 0)
            .map(|(i, _)| i).collect());
    }

    // ── Recent launches ───────────────────────────────────────────────────────
    if q.contains("last") || q.contains("recent") {
        let count = parse_count(&q, 10);
        let mut recent: Vec<usize> = apps.iter().enumerate()
            .filter(|(_, a)| a.last_launched.is_some())
            .map(|(i, _)| i)
            .collect();
        recent.sort_by(|a, b| apps[*b].last_launched.cmp(&apps[*a].last_launched));
        return Some(recent.into_iter().take(count).collect());
    }

    None
}

/// Score an app for ranking within search results.
/// Higher = better match. Combines match tier with usage habits.
pub fn score_app(app: &AppEntry, query: &str, now: u64) -> u32 {
    let lower_name = app.name.to_lowercase();
    let q = query.trim().to_lowercase();

    // Match tier — prefix always beats contains, contains beats fuzzy
    let tier_score: u32 = if lower_name == q {
        100_000  // exact
    } else if lower_name.starts_with(&q) {
        75_000   // prefix
    } else if lower_name.contains(&q) {
        50_000   // contains
    } else {
        25_000   // fuzzy
    };

    // Usage boost — launch count (capped at 1000 points)
    let usage_score = (app.launch_count * 10).min(1000);

    // Recency boost — used in last 7 days gets up to 500 points
    let recency_score = if let Some(last) = app.last_launched {
        let age = now.saturating_sub(last);
        if age < DAY { 500 }
        else if age < DAY * 3 { 300 }
        else if age < DAY * 7 { 100 }
        else { 0 }
    } else { 0 };

    tier_score + usage_score + recency_score
}
