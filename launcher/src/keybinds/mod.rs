// MIT License - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// src/keybinds/mod.rs
// Central keyboard shortcut handling for the launcher.
// All keybinds are routed through handle_key() which delegates to actions.rs.

pub mod actions;

pub use actions::handle_key;
