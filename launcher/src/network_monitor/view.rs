// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/network_monitor/view.rs
// Network monitor view - bandwidth/ping graphs and readout layout.

use cosmic::iced::{
    Element, Length,
    widget::{canvas, column, container, row, text},
};

use crate::network_monitor::{
    Message, NetworkState,
    graph::{DOWN_COLOR, UP_COLOR, PING_COLOR, JITTER_COLOR, NetGraph},
};

/// Renders the network monitor into the 140×70 widget box.
pub fn view(state: &NetworkState) -> Element<'_, Message> {
    // ── Sparkline graph ────────────────────────────────────────────────────
    let graph: Element<'_, Message> = canvas(NetGraph::new(
        state.bandwidth.down_history.clone(),
        state.bandwidth.up_history.clone(),
    ))
    .width(Length::Fill)
    .height(Length::Fixed(52.0))
    .into();

    // ── Stats row ──────────────────────────────────────────────────────────
    let down_label = text("↓").size(9).color(DOWN_COLOR);
    let down_val   = text(fmt_speed(state.bandwidth.down_kbps)).size(9).color(crate::ui::theme::get().text_steel);

    let up_label   = text("↑").size(9).color(UP_COLOR);
    let up_val     = text(fmt_speed(state.bandwidth.up_kbps)).size(9).color(crate::ui::theme::get().text_steel);

    let ping_label = text("ping").size(9).color(PING_COLOR);
    let ping_val   = text(fmt_ping(state.ping.ping_ms)).size(9).color(crate::ui::theme::get().text_steel);

    let jitter_label = text("jitr").size(9).color(JITTER_COLOR);
    let jitter_val   = text(fmt_jitter(state.ping.jitter_ms)).size(9).color(crate::ui::theme::get().text_steel);

    let stats = row![
        column![down_label,   down_val  ].spacing(1).width(Length::Fill),
        column![up_label,     up_val    ].spacing(1).width(Length::Fill),
        column![ping_label,   ping_val  ].spacing(1).width(Length::Fill),
        column![jitter_label, jitter_val].spacing(1).width(Length::Fill),
    ]
    .spacing(2);

    container(
        column![graph, stats].spacing(4)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding([4, 12, 6, 12])
    .into()
}

// ── Formatters ────────────────────────────────────────────────────────────────

fn fmt_speed(kbps: f32) -> String {
    if kbps >= 1024.0 {
        format!("{:.1}M", kbps / 1024.0)
    } else if kbps >= 1.0 {
        format!("{:.0}K", kbps)
    } else {
        "0K".to_string()
    }
}

fn fmt_ping(ms: f32) -> String {
    if ms > 0.0 { format!("{:.0}ms", ms) } else { "—".to_string() }
}

fn fmt_jitter(ms: f32) -> String {
    if ms > 0.0 { format!("±{:.0}", ms) } else { "±—".to_string() }
}

// === DONE ===
// view(): 140×70 layout — sparkline graph on top, 4-column stats row below :: done
// fmt_speed(): auto-scales K/M :: done
// fmt_ping() / fmt_jitter(): shows — before first measurement :: done