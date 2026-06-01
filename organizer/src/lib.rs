// MIT License - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// src/organizer/mod.rs
// File organizer — watches Downloads and suggests where files should move.
// Never moves anything without user permission.

pub mod rules;
pub mod scan;


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
    /// User approved a move suggestion — see issue #20
    #[allow(dead_code)]
    ApproveSuggestion(usize),
    /// User dismissed a suggestion — see issue #20
    #[allow(dead_code)]
    DismissSuggestion(usize),
}

impl OrganizerState {
    pub fn new() -> Self {
        let mut pending = load_pending();
        let existing_paths: std::collections::HashSet<_> = pending.iter()
            .map(|p| p.suggestion.from.clone()).collect();
        for s in scan::scan() {
            if !existing_paths.contains(&s.suggestion.from) {
                pending.push(s);
            }
        }
        save_pending(&pending);
        Self { pending }
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::FileDetected(path) => {
                if let Some(suggestion) = rules::suggest(&path) {
                    self.pending.push(PendingSuggestion { suggestion });
                    save_pending(&self.pending);
                }
            }
            Message::ApproveSuggestion(idx) => {
                if idx < self.pending.len() {
                    let s = self.pending.remove(idx);
                    // Create destination directory if needed
                    if let Some(parent) = s.suggestion.to.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    if let Err(_e) = std::fs::rename(&s.suggestion.from, &s.suggestion.to) {
                    }
                    save_pending(&self.pending);
                }
            }
            Message::DismissSuggestion(idx) => {
                if idx < self.pending.len() {
                    self.pending.remove(idx);
                    save_pending(&self.pending);
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

        // Keep watcher alive and poll on a background thread
        let (event_tx, event_rx) = std::sync::mpsc::channel::<std::path::PathBuf>();
        std::thread::spawn(move || {
            let _watcher = watcher;
            loop {
                if let Ok(Ok(event)) = notify_rx.recv() {
                    if matches!(event.kind, EventKind::Create(CreateKind::File) | EventKind::Create(CreateKind::Any) | EventKind::Modify(notify::event::ModifyKind::Name(notify::event::RenameMode::To))) {
                        for path in event.paths {
                            let _ = event_tx.send(path);
                        }
                    }
                }
            }
        });

        loop {
            match event_rx.try_recv() {
                Ok(path) => {
                    let _ = tx.try_send(Message::FileDetected(path));
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
            }
        }
    })
}

// ── Persistence ───────────────────────────────────────────────────────────────

fn pending_path() -> Option<std::path::PathBuf> {
    Some(dirs::data_local_dir()?.join("soulless").join("organizer_pending.json"))
}

fn save_pending(pending: &[PendingSuggestion]) {
    let Some(path) = pending_path() else { return };
    if let Some(parent) = path.parent() { let _ = std::fs::create_dir_all(parent); }
    let items: Vec<(String, String)> = pending.iter()
        .map(|p| (p.suggestion.from.to_string_lossy().to_string(),
                  p.suggestion.to.to_string_lossy().to_string()))
        .collect();
    if let Ok(json) = serde_json::to_string_pretty(&items) {
        let _ = std::fs::write(&path, json);
    }
}

fn load_pending() -> Vec<PendingSuggestion> {
    let Some(path) = pending_path() else { return vec![] };
    let Ok(text) = std::fs::read_to_string(&path) else { return vec![] };
    let Ok(items): Result<Vec<(String, String)>, _> = serde_json::from_str(&text) else { return vec![] };
    items.into_iter().filter_map(|(from, to)| {
        let from = std::path::PathBuf::from(&from);
        let to = std::path::PathBuf::from(&to);
        let reason = format!("{} looks like it belongs in {}",
            from.file_name()?.to_str()?,
            to.parent()?.display());
        Some(PendingSuggestion { suggestion: rules::MoveSuggestion { from, to, reason } })
    }).collect()
}

