// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// launcher/src/fps_monitor/monitors.rs
// Monitor census: one row per connected display, fed by compositor
// OutputEvents (Created / InfoUpdate / Removed). Concern one: build the
// roster and prove it on stderr. No view wiring yet.

use cosmic::iced::event::{self, wayland};
use cosmic::iced::{Event, Subscription};

// ── Data ─────────────────────────────────────────────────────────────────

/// One connected display, distilled from the compositor's output info.
#[derive(Debug, Clone)]
pub struct Monitor {
    /// Stable identity minted from the compositor's output handle. Created /
    /// InfoUpdate / Removed for one physical output all carry the same
    /// handle, so this string keys the census across all three — including
    /// Removed, which carries no info payload.
    pub key: String,
    /// wl_output global name (compositor-assigned id). 0 until known.
    /// Unused today; the debug-string key is the join column. Kept as metadata.
    #[allow(dead_code)]
    pub global_id: u32,
    /// Connector name ("DP-1") when provided, else "make model".
    pub name: String,
    /// Current-mode refresh in millihertz (164834 = 164.834 Hz). 0 = unknown.
    pub refresh_mhz: i32,
}

/// The census: every output the compositor has announced and not removed.
#[derive(Debug, Clone, Default)]
pub struct MonitorsState {
    pub monitors: Vec<Monitor>,
}

impl MonitorsState {
    /// Fold one compositor output event into the census.
    pub fn apply(&mut self, evt: wayland::OutputEvent, key: String) {
        match evt {
            wayland::OutputEvent::Created(info) => {
                let (global_id, name, refresh_mhz) = info
                    .as_ref()
                    .map(|i| {
                        (
                            i.id,
                            i.name
                                .clone()
                                .unwrap_or_else(|| format!("{} {}", i.make, i.model)),
                            i.modes
                                .iter()
                                .find(|m| m.current)
                                .map(|m| m.refresh_rate)
                                .unwrap_or(0),
                        )
                    })
                    .unwrap_or_else(|| (0, String::from("unknown"), 0));
                eprintln!(
                    "[MONITORS] created  {name} ({:.3} Hz) key={key}",
                    refresh_mhz as f64 / 1000.0
                );
                self.upsert(Monitor { key, global_id, name, refresh_mhz });
                self.census();
            }
            wayland::OutputEvent::InfoUpdate(i) => {
                let name = i
                    .name
                    .clone()
                    .unwrap_or_else(|| format!("{} {}", i.make, i.model));
                let refresh_mhz = i
                    .modes
                    .iter()
                    .find(|m| m.current)
                    .map(|m| m.refresh_rate)
                    .unwrap_or(0);
                eprintln!(
                    "[MONITORS] update   {name} ({:.3} Hz) key={key}",
                    refresh_mhz as f64 / 1000.0
                );
                self.upsert(Monitor { key, global_id: i.id, name, refresh_mhz });
                self.census();
            }
            wayland::OutputEvent::Removed => {
                self.monitors.retain(|m| m.key != key);
                eprintln!("[MONITORS] removed  key={key}");
                self.census();
            }
        }
    }

    /// Replace the row with the same key, or append a new one.
    fn upsert(&mut self, m: Monitor) {
        if let Some(slot) = self.monitors.iter_mut().find(|x| x.key == m.key) {
            *slot = m;
        } else {
            self.monitors.push(m);
        }
    }

    /// One-line roster dump after every change — the concern-one proof.
    fn census(&self) {
        let roster: Vec<String> = self
            .monitors
            .iter()
            .map(|m| format!("{} @ {:.3} Hz", m.name, m.refresh_mhz as f64 / 1000.0))
            .collect();
        eprintln!("[MONITORS] census   [{}]", roster.join(" | "));
    }
}

// ── Subscription ─────────────────────────────────────────────────────────

/// Raw compositor output events, tagged with a stable per-output key. The
/// key is the Debug form of the wl_output handle — same object, same
/// string — which lets Removed (no info payload) find its row.
pub fn subscription() -> Subscription<(wayland::OutputEvent, String)> {
    event::listen_raw(|ev, _status, _id| match ev {
        Event::PlatformSpecific(event::PlatformSpecific::Wayland(
            wayland::Event::Output(evt, output),
        )) => Some((evt, format!("{output:?}"))),
        _ => None,
    })
}

// === DONE ===
// Monitor row: key / global_id / name / refresh_mhz :: done
// MonitorsState: upsert-by-key census, Removed handled by key :: done
// subscription(): listen_raw on wayland Output events, keyed :: done
// stderr proof: created/update/removed + roster line :: done
