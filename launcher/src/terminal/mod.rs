// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/terminal/mod.rs
// Terminal page module: houses the scratch command box (terminal_box) and,
// later, the page-frame glue that gives it the four-slot footprint.

pub mod terminal_box;

#[allow(unused_imports)]
pub use terminal_box::{view, Line, Message, TerminalBox};
