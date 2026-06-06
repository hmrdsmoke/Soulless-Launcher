// GPL-3.0-or-later - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

use cosmic::iced::{
    self, Color,
    widget::canvas,
};

// ── Colours ───────────────────────────────────────────────────────────────────

pub const DOWN_COLOR:   Color = Color { r: 0.3, g: 0.8, b: 1.0, a: 1.0 };
pub const UP_COLOR:     Color = Color { r: 0.4, g: 1.0, b: 0.5, a: 1.0 };
pub const PING_COLOR:   Color = Color { r: 1.0, g: 0.85, b: 0.3, a: 1.0 };
pub const JITTER_COLOR: Color = Color { r: 1.0, g: 0.5,  b: 0.3, a: 1.0 };

// ── Widget ────────────────────────────────────────────────────────────────────

pub struct NetGraph {
    pub down: Vec<f32>,
    pub up:   Vec<f32>,
}

impl NetGraph {
    pub fn new(down: Vec<f32>, up: Vec<f32>) -> Self {
        NetGraph { down, up }
    }
}

impl<Msg> canvas::Program<Msg> for NetGraph {
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

        // Scale both lines to the same peak so they're comparable
        let peak = self.down.iter()
            .chain(self.up.iter())
            .cloned()
            .fold(1.0_f32, f32::max);

        draw_line(&mut frame, &self.down, bounds, peak, DOWN_COLOR);
        draw_line(&mut frame, &self.up,   bounds, peak, UP_COLOR);

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
// NetGraph: canvas::Program, draws background + two polylines (down + up) :: done
// Peak-normalised so both lines share the same Y scale :: done
// DOWN_COLOR / UP_COLOR / PING_COLOR / JITTER_COLOR exported for view.rs :: done