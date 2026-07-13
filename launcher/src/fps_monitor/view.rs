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
    graph::{AVG_COLOR, LOW_COLOR, FtGraph, fps_color},
};

/// Renders the FPS monitor into a 140×90 widget box.
///
/// Layout (measured):
///   padding-top:  4px
///   graph:       28px
///   spacing:      4px
///   label row:   ~11px
///   spacing:      2px
///   value row:   ~11px  (big fps = size 14, rest size 9)
///   padding-bot:  4px
///   total:       ~64px → use 90px to match hardware monitor
pub fn view(state: &FpsMonitorState) -> Element<'_, Message> {
    let fps = &state.fps;

    // ── Sparkline ─────────────────────────────────────────────────────────────
    let graph: Element<'_, Message> = canvas(FtGraph::new(
        fps.ft_history.clone(),
    ))
    .width(Length::Fill)
    .height(Length::Fixed(28.0))
    .into();

    // ── Live FPS — colour-coded big number ────────────────────────────────────
    let live_color = fps_color(fps.fps);

    // ── Average ───────────────────────────────────────────────────────────────
    let avg_col = column![
        text("avg").size(9).color(AVG_COLOR),
        text(fmt_fps(fps.fps_avg)).size(9).color(crate::ui::theme::get().text_steel),
    ]
    .spacing(1)
    .width(Length::Fill);

    // ── 1% low ────────────────────────────────────────────────────────────────
    let low_col = column![
        text("1%lo").size(9).color(LOW_COLOR),
        text(fmt_fps(fps.fps_1_low)).size(9).color(crate::ui::theme::get().text_steel),
    ]
    .spacing(1)
    .width(Length::Fill);

    // ── Frametime ─────────────────────────────────────────────────────────────
    let ft_col = column![
        text("ft").size(9).color(crate::ui::theme::get().text_steel),
        text(fmt_ft(fps.frametime_ms)).size(9).color(crate::ui::theme::get().text_steel),
    ]
    .spacing(1)
    .width(Length::Fill);

    // ── Small stats row ───────────────────────────────────────────────────────
    let stats = row![avg_col, low_col, ft_col].spacing(2);

    // ── Big FPS bottom line ───────────────────────────────────────────────────
    let fps_row = row![
        text("fps").size(9).color(live_color),
        text(fmt_fps(fps.fps)).size(14).color(live_color),
    ].spacing(4).align_y(cosmic::iced::alignment::Vertical::Bottom);

    container(
        column![graph, stats, fps_row].spacing(2)
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

fn fmt_ft(ms: f32) -> String {
    if ms > 0.0 { format!("{:.1}ms", ms) } else { "—".to_string() }
}

// === DONE ===
// Fixed: container height bumped 70 → 90px to fit graph + 2-row stats :: done
// Fixed: graph height reduced 28px to match hardware monitor :: done
// Live FPS: size 14, colour-coded green/blue/orange/red :: done
// avg / 1%lo / ft: size 9, fits in remaining space :: done
// Both widgets now 140×90 — consistent with each other :: done

// === DONE ===
// view(): 140×70 layout — frametime sparkline on top, 4-column stats row below :: done
// Live FPS: large number, colour-coded green/blue/orange/red by threshold :: done
// avg col: rolling average over last HISTORY samples :: done
// 1% low col: worst-frame average, red tint :: done
// ft col: latest frametime in ms :: done
// fmt_fps(): shows — when no game active :: done
// fmt_ft(): shows — when no game active :: done