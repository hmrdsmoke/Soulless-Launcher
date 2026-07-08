// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/fps_monitor/graph.rs
// Frametime graph canvas rendering and colour constants.

use cosmic::iced::{
    self, Color,
    widget::canvas,
};

// ── Colours ───────────────────────────────────────────────────────────────────

pub const FPS_COLOR:  Color = Color { r: 0.23, g: 0.44, b: 0.75, a: 1.0 }; // blue
pub const LOW_COLOR:  Color = Color { r: 0.75, g: 0.24, b: 0.18, a: 1.0 }; // red — 1% low marker
pub const AVG_COLOR:  Color = Color { r: 0.3,  g: 0.8,  b: 0.45, a: 1.0 }; // green — avg

/// Returns a colour for the live FPS number based on value.
pub fn fps_color(fps: f32) -> Color {
    if fps >= 120.0 {
        Color { r: 0.18, g: 0.75, b: 0.35, a: 1.0 } // bright green
    } else if fps >= 60.0 {
        Color { r: 0.23, g: 0.44, b: 0.75, a: 1.0 } // blue
    } else if fps >= 30.0 {
        Color { r: 0.75, g: 0.48, b: 0.17, a: 1.0 } // orange
    } else {
        Color { r: 0.75, g: 0.24, b: 0.18, a: 1.0 } // red
    }
}

// ── Widget ────────────────────────────────────────────────────────────────────

pub struct FtGraph {
    /// Frametime history in ms — plotted as a sparkline.
    pub frametime: Vec<f32>,
}

impl FtGraph {
    pub fn new(frametime: Vec<f32>) -> Self {
        FtGraph { frametime }
    }
}

impl<Msg> canvas::Program<Msg> for FtGraph {
    type State = ();

    fn draw(
        &self,
        _state:   &(),
        renderer: &iced::Renderer,
        _theme:   &iced::Theme,
        bounds:   iced::Rectangle,
        _cursor:  iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        frame.fill_rectangle(
            iced::Point::ORIGIN,
            bounds.size(),
            Color::from_rgba8(18, 18, 18, 0.75),
        );

        // Frametime: higher = worse, so peak is the worst frame
        let peak = self.frametime.iter()
            .cloned()
            .fold(1.0_f32, f32::max);

        draw_line(&mut frame, &self.frametime, bounds, peak, FPS_COLOR);

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
        // Frametime is inverted: higher frametime = worse = draw higher on canvas
        let y = v / peak * h;
        let y = y.clamp(0.0, h);

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
// FtGraph: canvas::Program, draws frametime sparkline :: done
// Frametime inverted on Y axis — spikes go up (worse = higher) :: done
// fps_color(): green >= 120, blue >= 60, orange >= 30, red < 30 :: done
// FPS_COLOR / LOW_COLOR / AVG_COLOR exported for view.rs :: done