// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/fps_monitor/view.rs
// FPS monitor view - frametime graph and readout layout.

use cosmic::iced::{
    Element, Length,
    widget::{canvas, column, container, row, text},
};

use crate::fps_monitor::{
    Message, FpsMonitorState,
    graph::{AVG_COLOR, LOW_COLOR, FPS_COLOR, FtGraph, fps_color},
};

/// Renders the FPS monitor into a 140×90 widget box.
///
/// Layout: frametime sparkline on top, then one census cell per
/// connected display — connector name / configured Hz / live fps.
pub fn view(state: &FpsMonitorState) -> Element<'_, Message> {
    let fps = &state.fps;

    // ── Sparkline ─────────────────────────────────────────────────────────────
    let graph: Element<'_, Message> = canvas(FtGraph::new(
        fps.ft_history.clone(),
    ))
    .width(Length::Fill)
    .height(Length::Fixed(28.0))
    .into();

    // ── Per-monitor cells (concern two: shell from the census) ────────────────
    // One column per connected display: connector name on top, configured
    // Hz from the census below, live per-screen fps as a dash until the
    // tagged frame ticks land (concern three).
    // Cells wrap two per row: below ~70px a cell's text clips, and the
    // box is 140 wide. 1-2 monitors = one row (unchanged look); 3-4 =
    // two rows. Verified via layout dummies before landing.
    let mut stats = column![].spacing(2);
    if state.monitors.monitors.is_empty() {
        stats = stats.push(row![column![
            text("outputs").size(sc(9.0)).color(AVG_COLOR),
            text("—").size(sc(9.0)).color(AVG_COLOR),
        ]
        .spacing(1)
        .width(Length::Fill)]);
    } else {
        for pair in state.monitors.monitors.chunks(2) {
            let mut r = row![].spacing(2);
            for m in pair {
                let live = m.live_fps.map(fmt_fps).unwrap_or_else(|| "—".into());
                let live_color = m.live_fps.map(fps_color).unwrap_or(LOW_COLOR);
                r = r.push(
                    column![
                        text(m.name.clone()).size(sc(8.0)).color(AVG_COLOR),
                        text(fmt_hz(m.refresh_mhz)).size(sc(8.0)).color(FPS_COLOR),
                        text(live).size(sc(8.0)).color(live_color),
                    ]
                    .spacing(1)
                    .width(Length::Fill),
                );
            }
            stats = stats.push(r);
        }
    }

    container(
        // Air between the sparkline and the cells -- the tall box affords it.
        column![graph, stats].spacing(8)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding([4, 12, 6, 12])
    .into()
}

// ── Formatters ────────────────────────────────────────────────────────────────

fn fmt_fps(fps: f32) -> String {
    if fps > 0.0 { format!("{:.0}", fps) } else { "—".to_string() }
}

/// Census refresh: millihertz → "144.0" (or — when unknown).
fn fmt_hz(mhz: i32) -> String {
    if mhz > 0 { format!("{:.1}", mhz as f64 / 1000.0) } else { "—".to_string() }
}

// === DONE ===
// Fixed: container height bumped 70 → 90px to fit graph + 2-row stats :: done
// Fixed: graph height reduced 28px to match hardware monitor :: done
// big fps control line: cut after per-monitor cells validated against it :: done
// per-monitor cells: name / cfg Hz / live fps, wrapped 2-per-row :: done
// Both widgets now 140×90 — consistent with each other :: done

// === DONE ===
// view(): 140×70 layout — frametime sparkline on top, 4-column stats row below :: done
// Live FPS: large number, colour-coded green/blue/orange/red by threshold :: done
// avg col: rolling average over last HISTORY samples :: done
// 1% low col: worst-frame average, red tint :: done
// ft col: latest frametime in ms :: done
// fmt_fps(): shows — when no game active :: done
// fmt_ft(): shows — when no game active :: done
fn sc(base: f32) -> f32 { base * crate::ui::theme::get().widget_scale }
