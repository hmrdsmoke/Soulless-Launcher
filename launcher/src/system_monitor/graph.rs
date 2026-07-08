// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/system_monitor/graph.rs
// System graph canvas rendering and colour constants.

use cosmic::iced::{
    self, Color,
    widget::canvas,
};

// ── Colours ───────────────────────────────────────────────────────────────────

pub const CPU_COLOR:  Color = Color { r: 0.4, g: 0.8, b: 1.0, a: 1.0 }; // blue
pub const RAM_COLOR:  Color = Color { r: 0.6, g: 1.0, b: 0.4, a: 1.0 }; // green
pub const GPU_COLOR:  Color = Color { r: 1.0, g: 0.5, b: 0.9, a: 1.0 }; // pink
pub const DISK_COLOR: Color = Color { r: 1.0, g: 0.8, b: 0.3, a: 1.0 }; // amber

// ── Widget ────────────────────────────────────────────────────────────────────

pub struct SysGraph {
    pub cpu:  Vec<f32>,
    pub ram:  Vec<f32>,
    pub gpu:  Vec<f32>,
    pub disk: Vec<f32>,
}

impl SysGraph {
    pub fn new(cpu: Vec<f32>, ram: Vec<f32>, gpu: Vec<f32>, disk: Vec<f32>) -> Self {
        SysGraph { cpu, ram, gpu, disk }
    }
}

impl<Msg> canvas::Program<Msg> for SysGraph {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // Dark background
        frame.fill_rectangle(
            iced::Point::ORIGIN,
            bounds.size(),
            Color::from_rgba8(18, 18, 18, 0.75),
        );

        // All four lines share the same 0–100 scale (they're all percentages)
        let peak = 100.0_f32;

        draw_line(&mut frame, &self.cpu,  bounds, peak, CPU_COLOR);
        draw_line(&mut frame, &self.ram,  bounds, peak, RAM_COLOR);
        draw_line(&mut frame, &self.gpu,  bounds, peak, GPU_COLOR);
        draw_line(&mut frame, &self.disk, bounds, peak, DISK_COLOR);

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

// === DONE ===
// SysGraph: canvas::Program, draws background + four polylines (cpu/ram/gpu/disk) :: done
// Fixed 0–100 scale since all values are percentages :: done
// CPU_COLOR / RAM_COLOR / GPU_COLOR / DISK_COLOR exported for view.rs :: done