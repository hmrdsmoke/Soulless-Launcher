// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/hardware_monitor/graph.rs
// Hardware graph canvas rendering and colour constants.

use cosmic::iced::{
    self, Color,
    widget::canvas,
};

// ── Colours ───────────────────────────────────────────────────────────────────

pub const CPU_COLOR:  Color = Color { r: 0.23, g: 0.44, b: 0.75, a: 1.0 }; // #3a70bf — blue
pub const GPU_COLOR:  Color = Color { r: 0.18, g: 0.55, b: 0.34, a: 1.0 }; // #2e8b57 — green
pub const RAM_COLOR:  Color = Color { r: 0.55, g: 0.37, b: 0.18, a: 1.0 }; // #8b5e2e — amber
pub const TEMP_HOT:   Color = Color { r: 0.75, g: 0.24, b: 0.18, a: 1.0 }; // #c03d2e — red
pub const TEMP_WARM:  Color = Color { r: 0.75, g: 0.48, b: 0.17, a: 1.0 }; // #bf7a2c — orange

// ── Sparkline widget ──────────────────────────────────────────────────────────

pub struct HwGraph {
    pub cpu: Vec<f32>,
    pub gpu: Vec<f32>,
    pub ram: Vec<f32>,
}

impl HwGraph {
    pub fn new(cpu: Vec<f32>, gpu: Vec<f32>, ram: Vec<f32>) -> Self {
        HwGraph { cpu, gpu, ram }
    }
}

impl<Msg> canvas::Program<Msg> for HwGraph {
    type State = ();

    fn draw(
        &self,
        _state:    &(),
        renderer:  &iced::Renderer,
        _theme:    &iced::Theme,
        bounds:    iced::Rectangle,
        _cursor:   iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // Dark background
        frame.fill_rectangle(
            iced::Point::ORIGIN,
            bounds.size(),
            Color::from_rgba8(18, 18, 18, 0.75),
        );

        // All three lines share the same 0–100 scale (they're all percentages)
        let peak = 100.0_f32;

        draw_line(&mut frame, &self.cpu, bounds, peak, CPU_COLOR);
        draw_line(&mut frame, &self.gpu, bounds, peak, GPU_COLOR);
        draw_line(&mut frame, &self.ram, bounds, peak, RAM_COLOR);

        vec![frame.into_geometry()]
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn draw_line(
    frame:  &mut canvas::Frame,
    data:   &[f32],
    bounds: iced::Rectangle,
    peak:   f32,
    color:  Color,
) {
    if data.len() < 2 {
        return;
    }

    let w    = bounds.width;
    let h    = bounds.height;
    let step = w / (data.len() - 1) as f32;

    let mut path = canvas::path::Builder::new();

    for (i, &v) in data.iter().enumerate() {
        let x = i as f32 * step;
        let y = h - (v / peak * h).clamp(0.0, h);

        if i == 0 {
            path.move_to(iced::Point::new(x, y));
        } else {
            path.line_to(iced::Point::new(x, y));
        }
    }

    frame.stroke(
        &path.build(),
        canvas::Stroke::default()
            .with_color(color)
            .with_width(1.5),
    );
}

/// Returns a temperature colour: red if hot, orange if warm, else default.
pub fn temp_color(temp: Option<f32>, default: Color) -> Color {
    match temp {
        Some(t) if t >= 80.0 => TEMP_HOT,
        Some(t) if t >= 65.0 => TEMP_WARM,
        _ => default,
    }
}

// === DONE ===
// HwGraph: canvas::Program, draws background + CPU/GPU/RAM polylines :: done
// All three lines on shared 0-100 scale (all are percentages) :: done
// CPU_COLOR / GPU_COLOR / RAM_COLOR exported for view.rs :: done
// temp_color() helper: red >= 80°C, orange >= 65°C :: done