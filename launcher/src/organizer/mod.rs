// MIT License - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// src/organizer/mod.rs
// File organizer — watches Downloads and suggests where files should move.
// Never moves anything without user permission.

pub mod rules;

use std::path::PathBuf;

/// A pending move suggestion waiting for user approval.
#[derive(Debug, Clone)]
pub struct PendingSuggestion {
    pub suggestion: rules::MoveSuggestion,
}

/// Organizer state.
#[derive(Debug, Clone, Default)]
pub struct OrganizerState {
    /// Suggestions waiting for user approval
    pub pending: Vec<PendingSuggestion>,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// A new file appeared in Downloads
    FileDetected(PathBuf),
    /// User approved a move suggestion
    ApproveSuggestion(usize),
    /// User dismissed a suggestion
    DismissSuggestion(usize),
}

impl OrganizerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::FileDetected(path) => {
                if let Some(suggestion) = rules::suggest(&path) {
                    self.pending.push(PendingSuggestion { suggestion });
                }
            }
            Message::ApproveSuggestion(idx) => {
                if idx < self.pending.len() {
                    let s = self.pending.remove(idx);
                    // Create destination directory if needed
                    if let Some(parent) = s.suggestion.to.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    if let Err(e) = std::fs::rename(&s.suggestion.from, &s.suggestion.to) {
                        eprintln!("organizer: move failed: {}", e);
                    }
                }
            }
            Message::DismissSuggestion(idx) => {
                if idx < self.pending.len() {
                    self.pending.remove(idx);
                }
            }
        }
    }
}

/// Returns a subscription that watches ~/Downloads for new files.
pub fn subscription() -> cosmic::iced::Subscription<Message> {
    cosmic::iced::Subscription::run(watcher_stream)
}

fn watcher_stream() -> impl cosmic::iced::futures::Stream<Item = Message> {
    use cosmic::iced::stream;
    stream::channel(16, |mut tx: cosmic::iced::futures::channel::mpsc::Sender<Message>| async move {
        use notify::{RecursiveMode, Watcher, EventKind};
        use notify::event::CreateKind;

        let downloads = dirs::download_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap().join("Downloads"));

        let (notify_tx, notify_rx) = std::sync::mpsc::channel();

        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = notify_tx.send(res);
        }).expect("organizer: failed to create watcher");

        watcher.watch(&downloads, RecursiveMode::NonRecursive)
            .expect("organizer: failed to watch Downloads");

        loop {
            if let Ok(Ok(event)) = notify_rx.recv() {
                if matches!(event.kind, EventKind::Create(CreateKind::File)) {
                    for path in event.paths {
                        let _ = tx.try_send(Message::FileDetected(path));
                    }
                }
            }
        }
    })
}
