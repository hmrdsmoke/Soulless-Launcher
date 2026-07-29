// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/terminal/terminal_box.rs
//
// ── Scratch command box ───────────────────────────────────────────────────────
//
// A throwaway one-liner box: type a command, see its output, get on with your
// day — pkill the launcher after a make install without dirtying the real
// working terminal. Deliberately NOT a terminal emulator: no PTY, no VT
// parsing, no interactive programs. `sh -c` in, captured stdout/stderr out.
//
// Lifecycle contract (page system):
//   - Page switches preserve everything (state lives in the app struct).
//   - Window dismiss calls reset(): scrollback cleared, in-flight results
//     invalidated. Fresh box every summon, same as search/vault.
//
// Runaway guard: every command runs under `timeout 30`. A fat-fingered
// bare `ping` dies on its own — no kill button, no child-handle plumbing.
//
// No subscription — this module is input-driven only. It will not tick.

#![allow(dead_code)] // wired in by the page frame; allow until then

use cosmic::iced::widget::{column, container, row, scrollable, text, text_input};
use cosmic::iced::{Color, Element, Font, Length, Task};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Scrollback cap in lines. Oldest lines drop first. Keeps a `find /`
/// fat-finger from holding megabytes for the daemon's life.
const MAX_LINES: usize = 400;

/// Hard wall-clock cap per command, enforced by coreutils `timeout`.
const TIMEOUT_SECS: &str = "30";

/// Prompt glyph shown on echoed command lines and the input row.
const PROMPT: &str = "❯";

/// Command-echo lines render in the brand yellow (same family as the page
/// dots). Local const like the monitors' graph colors — functional
/// signalling, deliberately not themed.
const PROMPT_COLOR: Color = Color::from_rgb(1.0, 0.78, 0.16);

/// Muted red for stderr lines — visually separates noise from signal.
const STDERR_COLOR: Color = Color::from_rgb(0.86, 0.42, 0.42);

const TEXT_SIZE: f32 = 11.0;

// ── Messages ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    Submit,
    /// (generation, lines) — results from a stale generation (pre-reset or
    /// superseded) are dropped on arrival.
    Finished(u64, Vec<Line>),
}

// ── State ─────────────────────────────────────────────────────────────────────

/// One scrollback line, tagged by origin so the view can color it.
#[derive(Debug, Clone)]
pub enum Line {
    /// The echoed command itself: "❯ pkill -f soulless-launcher"
    Cmd(String),
    Out(String),
    Err(String),
    /// Status notes from the box itself (timeout, spawn failure).
    Note(String),
}

pub struct TerminalBox {
    pub input: String,
    scrollback: Vec<Line>,
    running: bool,
    /// Bumped on every submit AND every reset. A finished command only lands
    /// if its generation still matches — dismissing the launcher mid-command
    /// means the orphan's output arrives, mismatches, and vanishes.
    generation: u64,
}

impl TerminalBox {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            scrollback: Vec::new(),
            running: false,
            generation: 0,
        }
    }

    /// Dismiss teardown — mirrors search.reset_to_default() / vault.lock().
    /// The next summon gets a virgin box; any in-flight command's output is
    /// invalidated by the generation bump.
    pub fn reset(&mut self) {
        self.input.clear();
        self.scrollback.clear();
        self.running = false;
        self.generation = self.generation.wrapping_add(1);
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::InputChanged(s) => {
                self.input = s;
                Task::none()
            }
            Message::Submit => {
                let cmd = self.input.trim().to_string();
                if cmd.is_empty() || self.running {
                    return Task::none();
                }
                // Local special case, no spawn: wipe the scrollback.
                if cmd == "clear" {
                    self.input.clear();
                    self.scrollback.clear();
                    return Task::none();
                }
                self.input.clear();
                self.push(Line::Cmd(cmd.clone()));
                self.running = true;
                self.generation = self.generation.wrapping_add(1);
                let generation = self.generation;
                Task::batch([run(cmd, generation), snap_to_bottom()])
            }
            Message::Finished(generation, lines) => {
                if generation != self.generation {
                    return Task::none(); // stale: superseded or post-reset
                }
                self.running = false;
                for l in lines {
                    self.push(l);
                }
                snap_to_bottom()
            }
        }
    }

    fn push(&mut self, line: Line) {
        self.scrollback.push(line);
        if self.scrollback.len() > MAX_LINES {
            let overflow = self.scrollback.len() - MAX_LINES;
            self.scrollback.drain(..overflow);
        }
    }
}

// ── Command execution ─────────────────────────────────────────────────────────

/// Spawn `timeout 30 sh -c <cmd>` off-thread, capture both pipes, hand the
/// result back as a message. spawn_blocking keeps Command::output()'s wait
/// off the UI thread — a slow command never freezes the launcher, and
/// dismiss stays responsive (the escape hatch actually escapes).
///
/// NOTE: .output() captures stdout and stderr as separate pipes, so their
/// interleaving is lost — stdout renders first, then stderr. Fine for
/// glance-and-go; a real terminal this is not.
fn run(cmd: String, generation: u64) -> Task<Message> {
    Task::perform(
        async move {
            let result = tokio::task::spawn_blocking(move || {
                std::process::Command::new("timeout")
                    .args([TIMEOUT_SECS, "sh", "-c", &cmd])
                    .output()
            })
            .await;

            let mut lines: Vec<Line> = Vec::new();
            match result {
                Ok(Ok(out)) => {
                    for l in String::from_utf8_lossy(&out.stdout).lines() {
                        lines.push(Line::Out(l.to_string()));
                    }
                    for l in String::from_utf8_lossy(&out.stderr).lines() {
                        lines.push(Line::Err(l.to_string()));
                    }
                    match out.status.code() {
                        Some(124) => lines.push(Line::Note(format!(
                            "(timed out after {}s)",
                            TIMEOUT_SECS
                        ))),
                        Some(0) | None => {}
                        Some(n) => lines.push(Line::Note(format!("(exit {})", n))),
                    }
                }
                Ok(Err(e)) => lines.push(Line::Note(format!("(failed to spawn: {})", e))),
                Err(e) => lines.push(Line::Note(format!("(worker error: {})", e))),
            }
            lines
        },
        move |lines| Message::Finished(generation, lines),
    )
}

// ── View ──────────────────────────────────────────────────────────────────────

fn scroll_id() -> cosmic::widget::Id {
    cosmic::widget::Id::new("terminal-box-scroll")
}

fn snap_to_bottom() -> Task<Message> {
    scrollable::snap_to(
        scroll_id(),
        scrollable::RelativeOffset { x: Some(0.0), y: Some(1.0) },
    )
}

/// Fills whatever container the page frame gives it — the frame owns the
/// chrome (border, background, footprint), same contract as the monitors.
pub fn view(state: &TerminalBox) -> Element<'_, Message> {
    let mut lines = column![].spacing(1);
    for line in &state.scrollback {
        let t = match line {
            Line::Cmd(s) => text(format!("{} {}", PROMPT, s))
                .size(TEXT_SIZE)
                .font(Font::MONOSPACE)
                .color(PROMPT_COLOR),
            Line::Out(s) => text(s.clone()).size(TEXT_SIZE).font(Font::MONOSPACE),
            Line::Err(s) => text(s.clone())
                .size(TEXT_SIZE)
                .font(Font::MONOSPACE)
                .color(STDERR_COLOR),
            Line::Note(s) => text(s.clone())
                .size(TEXT_SIZE)
                .font(Font::MONOSPACE)
                .color(Color::from_rgb(0.55, 0.55, 0.55)),
        };
        lines = lines.push(t);
    }

    let output = scrollable(lines)
        .id(scroll_id())
        .width(Length::Fill)
        .height(Length::Fill);

    let prompt_glyph = text(PROMPT)
        .size(TEXT_SIZE)
        .font(Font::MONOSPACE)
        .color(PROMPT_COLOR);

    let input = text_input(if state.running { "running…" } else { "command" }, &state.input)
        .on_input(Message::InputChanged)
        .on_submit(Message::Submit)
        .size(TEXT_SIZE)
        .font(Font::MONOSPACE);

    let prompt_row = row![prompt_glyph, input]
        .spacing(6)
        .align_y(cosmic::iced::Alignment::Center);

    container(column![output, prompt_row].spacing(6))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(8)
        .into()
}