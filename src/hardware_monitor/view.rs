// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

use cosmic::iced::{
    Element, Length,
    widget::{canvas, column, container, row, text},
};

use crate::hardware_monitor::{
    Message, HardwareMonitorState,
    graph::{CPU_COLOR, GPU_COLOR, RAM_COLOR, HwGraph, temp_color},
};

/// Renders the hardware monitor into a 140×90 widget box.
///
/// Layout (measured):
///   padding-top:  4px
///   graph:       28px
///   spacing:      4px
///   label row:   ~11px  (size 9 text)
///   spacing:      2px
///   value row:   ~11px  (size 9 text)
///   spacing:      2px
///   sub row:     ~11px  (size 9 text)
///   padding-bot:  4px
///   total:       ~77px → use 90px for breathing room
pub fn view(state: &HardwareMonitorState) -> Element<'_, Message> {
    let hw = &state.hw;

    // ── Sparkline ─────────────────────────────────────────────────────────────
    let graph: Element<'_, Message> = canvas(HwGraph::new(
        hw.cpu_history.clone(),
        hw.gpu_history.clone(),
        hw.ram_history.clone(),
    ))
    .width(Length::Fill)
    .height(Length::Fixed(28.0))
    .into();

    // ── CPU column ────────────────────────────────────────────────────────────
    let cpu_temp_color = temp_color(hw.cpu_temp, CPU_COLOR);

    let cpu_col = column![
        text("cpu").size(9).color(CPU_COLOR),
        text(fmt_pct(hw.cpu_usage)).size(9),
        text(fmt_freq(hw.cpu_freq_mhz)).size(9).color(cpu_temp_color),
    ]
    .spacing(1)
    .width(Length::Fill);

    // ── GPU column ────────────────────────────────────────────────────────────
    let gpu_temp_color = temp_color(hw.gpu_temp.map(|t| t as f32), GPU_COLOR);

    let gpu_usage_str = hw.gpu_usage
        .map(|u| fmt_pct(u as f32))
        .unwrap_or_else(|| "—".to_string());

    let gpu_temp_str = hw.gpu_temp
        .map(fmt_temp_u32)
        .unwrap_or_else(|| "—".to_string());

    let gpu_col = column![
        text("gpu").size(9).color(GPU_COLOR),
        text(gpu_usage_str).size(9),
        text(gpu_temp_str).size(9).color(gpu_temp_color),
    ]
    .spacing(1)
    .width(Length::Fill);

    // ── RAM column ────────────────────────────────────────────────────────────
    let ram_col = column![
        text("ram").size(9).color(RAM_COLOR),
        text(fmt_pct(hw.ram_pct)).size(9),
        text(fmt_ram(hw.ram_used_mb, hw.ram_total_mb)).size(9),
    ]
    .spacing(1)
    .width(Length::Fill);

    // ── Stats row ─────────────────────────────────────────────────────────────
    let stats = row![cpu_col, gpu_col, ram_col].spacing(2);

    container(
        column![graph, stats].spacing(4)
    )
    .width(Length::Fixed(140.0))
    .height(Length::Fixed(90.0))
    .padding([4, 6])
    .style(|_| container::Style {
        background: Some(
            cosmic::iced::Color::from_rgba8(45, 45, 45, 0.90).into(),
        ),
        border: cosmic::iced::Border {
            radius: 12.0.into(),
            width:  1.0,
            color:  cosmic::iced::Color::from_rgb8(70, 70, 70),
        },
        ..Default::default()
    })
    .into()
}

// ── Formatters ────────────────────────────────────────────────────────────────

fn fmt_pct(v: f32) -> String {
    format!("{:.0}%", v)
}

fn fmt_freq(mhz: u64) -> String {
    if mhz >= 1000 {
        format!("{:.1}G", mhz as f32 / 1000.0)
    } else {
        format!("{}M", mhz)
    }
}

fn fmt_temp_u32(t: u32) -> String {
    format!("{}°", t)
}

fn fmt_ram(used_mb: u64, total_mb: u64) -> String {
    if total_mb >= 4096 {
        format!("{:.0}/{:.0}G",
            used_mb  as f32 / 1024.0,
            total_mb as f32 / 1024.0,
        )
    } else {
        format!("{}M", used_mb)
    }
}

// === DONE ===
// Fixed: container height bumped 70 → 90px to fit graph + 3-row stats :: done
// Fixed: graph height reduced 36 → 28px to free space for text rows :: done
// cpu col: label / usage% / freq (temp-coloured) :: done
// gpu col: label / usage% / temp° (temp-coloured, — if no NVML) :: done
// ram col: label / usage% / used/total :: done

// === DONE ===
// view(): 140×70 layout — sparkline on top, 3-column stats row below :: done
// cpu col: label / usage% / freq (coloured by temp) :: done
// gpu col: label / usage% / temp (coloured by heat, — if NVML unavailable) :: done
// ram col: label / usage% / used/total :: done
// temp_color() wired — hot red, warm orange, cool stays accent colour :: done
// fmt_freq(): auto-scales MHz → GHz :: done
// fmt_ram(): auto-scales MB → GB :: done