// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/system_monitor/view.rs
// System monitor view - resource graphs and readout layout.

use cosmic::iced::{
    Element, Length,
    widget::{canvas, column, container, row, text},
};

use crate::system_monitor::{
    Message, SystemState,
    graph::{CPU_COLOR, RAM_COLOR, GPU_COLOR, DISK_COLOR, SysGraph},
};

/// Renders the system monitor into the 140×70 widget box.
pub fn view(state: &SystemState) -> Element<'_, Message> {
    // ── Sparkline graph ────────────────────────────────────────────────────
    let graph: Element<'_, Message> = canvas(SysGraph::new(
        state.stats.cpu_history.clone(),
        state.stats.ram_history.clone(),
        state.stats.gpu_history.clone(),
        state.stats.disk_history.clone(),
    ))
    .width(Length::Fill)
    .height(Length::Fixed(52.0))
    .into();

    // ── Stats row ──────────────────────────────────────────────────────────
    let cpu_label  = text("CPU").size(sc(9.0)).color(CPU_COLOR);
    let cpu_val    = text(fmt_pct(state.stats.cpu_pct)).size(sc(9.0)).color(CPU_COLOR);

    let ram_label  = text("RAM").size(sc(9.0)).color(RAM_COLOR);
    let ram_val    = text(fmt_pct(state.stats.ram_pct)).size(sc(9.0)).color(RAM_COLOR);

    let gpu_label  = text("GPU").size(sc(9.0)).color(GPU_COLOR);
    let gpu_val    = text(fmt_opt_pct(state.stats.gpu_pct)).size(sc(9.0)).color(GPU_COLOR);

    let disk_label = text("DSK").size(sc(9.0)).color(DISK_COLOR);
    let disk_val   = text(fmt_pct(state.stats.disk_pct)).size(sc(9.0)).color(DISK_COLOR);

    let stats = row![
        column![cpu_label,  cpu_val ].spacing(1).width(Length::Fill),
        column![ram_label,  ram_val ].spacing(1).width(Length::Fill),
        column![gpu_label,  gpu_val ].spacing(1).width(Length::Fill),
        column![disk_label, disk_val].spacing(1).width(Length::Fill),
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

fn fmt_pct(pct: f32) -> String {
    format!("{:.0}%", pct)
}

/// Percentage that may be unavailable — no readable GPU shows a dash rather
/// than 0%, which would be indistinguishable from an idle card.
fn fmt_opt_pct(pct: Option<f32>) -> String {
    pct.map(|p| format!("{:.0}%", p))
        .unwrap_or_else(|| "—".to_string())
}

// === DONE ===
// fmt_opt_pct(): GPU shows a dash when there's no readable source :: done

// === DONE ===
// view(): 140×70 layout — sparkline graph on top, 4-column stats row below :: done
// fmt_pct(): formats percentage value :: done
fn sc(base: f32) -> f32 { base * crate::ui::theme::get().widget_scale }
