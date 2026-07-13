// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/hardware_monitor/view.rs
// Hardware monitor view - temperature graphs and readout layout.

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
/// Shows temperatures and clock frequencies only —
/// usage % is handled by the system monitor widget.
///
/// Layout:
///   [ temp sparkline — CPU + GPU temp over time    ]
///   [ CPU col         | GPU col      | RAM col     ]
///   [ temp / freq     | temp / core  | freq MHz    ]
pub fn view(state: &HardwareMonitorState) -> Element<'_, Message> {
    let hw = &state.hw;

    // ── Sparkline — CPU and GPU temp histories ────────────────────────────────
    let graph: Element<'_, Message> = canvas(HwGraph::new(
        hw.cpu_history.clone(),
        hw.gpu_history.clone(),
        vec![],
    ))
    .width(Length::Fill)
    .height(Length::Fixed(28.0))
    .into();

    // ── CPU column ────────────────────────────────────────────────────────────
    let cpu_temp_color = temp_color(hw.cpu_temp_c, CPU_COLOR);

    let cpu_temp_str = hw.cpu_temp_c
        .map(|t| format!("{:.0}°C", t))
        .unwrap_or_else(|| "—".to_string());

    let cpu_col = column![
        text("cpu").size(9).color(CPU_COLOR),
        text(cpu_temp_str).size(9).color(cpu_temp_color),
        text(fmt_cpu_freq(hw.cpu_freq_mhz)).size(9).color(CPU_COLOR),
    ]
    .spacing(1)
    .width(Length::Fill);

    // ── GPU column ────────────────────────────────────────────────────────────
    let gpu_temp_color = temp_color(hw.gpu_temp_c.map(|t| t as f32), GPU_COLOR);

    let gpu_temp_str = hw.gpu_temp_c
        .map(|t| format!("{}°C", t))
        .unwrap_or_else(|| "—".to_string());

    let gpu_clock_str = hw.gpu_clock_mhz
        .map(|c| format!("{} MHz", c))
        .unwrap_or_else(|| "—".to_string());

    let gpu_col = column![
        text("gpu").size(9).color(GPU_COLOR),
        text(gpu_temp_str).size(9).color(gpu_temp_color),
        text(gpu_clock_str).size(9).color(GPU_COLOR),
    ]
    .spacing(1)
    .width(Length::Fill);

    // ── RAM column ────────────────────────────────────────────────────────────
    let ram_freq_str = hw.ram_freq_mhz
        .map(|f| format!("{} MHz", f))
        .unwrap_or_else(|| "—".to_string());

    let ram_col = column![
        text("ram").size(9).color(RAM_COLOR),
        text(ram_freq_str).size(9).color(RAM_COLOR),
        text("").size(9),
    ]
    .spacing(1)
    .width(Length::Fill);

    // ── Stats row ─────────────────────────────────────────────────────────────
    let stats = row![cpu_col, gpu_col, ram_col].spacing(2);

    container(
        column![graph, stats].spacing(4)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding([4, 12, 6, 12])
    .into()
}

// ── Formatters ────────────────────────────────────────────────────────────────

/// CPU freq: show GHz if >= 1000 MHz, else MHz
fn fmt_cpu_freq(mhz: u64) -> String {
    if mhz >= 1000 {
        format!("{:.2} GHz", mhz as f32 / 1000.0)
    } else if mhz > 0 {
        format!("{} MHz", mhz)
    } else {
        "—".to_string()
    }
}

// === DONE ===
// MHz spelled out in full for GPU clock and RAM freq :: done
// CPU freq auto-scales: GHz if >= 1000 MHz, MHz otherwise :: done
// cpu col: label / temp°C (heat-coloured) / freq :: done
// gpu col: label / temp°C (heat-coloured) / core clock MHz :: done
// ram col: label / freq MHz (dmidecode, static) :: done

// === DONE ===
// Reworked: usage% removed — shows temp + freq only :: done
// cpu col: label / temp°C (heat-coloured) / avg freq :: done
// gpu col: label / temp°C (heat-coloured) / core clock MHz :: done
// ram col: label / freq MHz (DMI, static) :: done
// Sparkline tracks CPU + GPU temp history (not usage) :: done
// RAM col has blank third row to keep column heights aligned :: done

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