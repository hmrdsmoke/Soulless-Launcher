// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/config/theme_loader.rs
// Loads ~/.config/soulless/theme.ron: PARTIAL overrides on top of the baked-in
// ship look. Colors are 6-char Pop!_OS-style hex strings ("#E0E3E8") — hex sets
// RGB, alpha is inherited from the token's default (shadows stay translucent).
// Numerics are plain numbers. Missing file = silent defaults. Bad field =
// warn on stderr, keep that field's default. Load once at startup;
// restart the launcher to apply changes.

use crate::ui::theme::ThemeColors;
use cosmic::iced::Color;
use serde::Deserialize;

/// Mirror of ThemeColors where every field is optional — this is what a user's
/// theme.ron deserializes into. serde(default) makes every field omittable, so
/// a one-line theme.ron overriding a single color is valid.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ThemeFile {
    text_steel: Option<String>,
    text_ink: Option<String>,
    drawer_title: Option<String>,
    drawer_hint: Option<String>,
    window_bg: Option<String>,
    window_corner_radius: Option<f32>,
    window_gap: Option<f32>,
    window_border: Option<String>,
    window_border_width: Option<f32>,
    steel_top: Option<String>,
    steel_mid_a: Option<String>,
    steel_mid_b: Option<String>,
    steel_bottom: Option<String>,
    steel_border: Option<String>,
    steel_corner_radius: Option<f32>,
    steel_shadow_color: Option<String>,
    steel_text: Option<String>,
    steel_vertical_inset: Option<f32>,
    right_panel_bg: Option<String>,
    right_panel_border: Option<String>,
    right_panel_corner_radius: Option<f32>,
    widget_bg: Option<String>,
    widget_border: Option<String>,
    widget_corner_radius: Option<f32>,
    widget_scale: Option<f32>,
    widget_height: Option<f32>,
    widget_height_tall: Option<f32>,
    widget_spacing: Option<u16>,
    drawer_btn_bg: Option<String>,
    drawer_btn_hover: Option<String>,
    drawer_btn_active: Option<String>,
    drawer_btn_border: Option<String>,
    drawer_btn_text: Option<String>,
    drawer_btn_text_hover: Option<String>,
}

/// ~/.config/soulless/theme.ron (sibling of config.ron).
pub fn theme_path() -> Option<std::path::PathBuf> {
    crate::config::config_path()
        .parent()
        .map(|dir| dir.join("theme.ron"))
}

/// Parse a Pop!_OS-style 6-char hex color: "#E0E3E8" (leading '#' optional).
fn parse_hex(s: &str) -> Option<(f32, f32, f32)> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some((r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0))
}

/// Overlay a hex override onto a color, keeping the default's alpha.
fn apply_color(dst: &mut Color, src: &Option<String>, name: &str) {
    if let Some(hex) = src {
        match parse_hex(hex) {
            Some((r, g, b)) => {
                dst.r = r;
                dst.g = g;
                dst.b = b;
            }
            None => eprintln!(
                "[theme] ignoring `{name}`: \"{hex}\" is not a 6-char hex color (e.g. \"#E0E3E8\")"
            ),
        }
    }
}

fn apply_f32(dst: &mut f32, src: &Option<f32>) {
    if let Some(v) = src {
        *dst = *v;
    }
}

/// Load the runtime theme: ship defaults, with theme.ron laid on top if present.
pub fn load() -> ThemeColors {
    let mut theme = ThemeColors::default();

    let Some(path) = theme_path() else { return theme };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return theme; // no theme.ron: ship look, silently
    };

    // RON demands explicit Some(...) around Option fields by default; enable
    // implicit_some so user theme files stay clean bare values ("#0A3D0A").
    let ron_opts = ron::Options::default()
        .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);
    let file: ThemeFile = match ron_opts.from_str(&text) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[theme] {} failed to parse ({e}); using ship defaults", path.display());
            return theme;
        }
    };

    apply_color(&mut theme.text_steel, &file.text_steel, "text_steel");
    apply_color(&mut theme.text_ink, &file.text_ink, "text_ink");
    apply_f32(&mut theme.window_gap, &file.window_gap);
    apply_color(&mut theme.drawer_title, &file.drawer_title, "drawer_title");
    apply_color(&mut theme.drawer_hint, &file.drawer_hint, "drawer_hint");
    apply_color(&mut theme.window_bg, &file.window_bg, "window_bg");
    apply_f32(&mut theme.window_corner_radius, &file.window_corner_radius);
    apply_color(&mut theme.window_border, &file.window_border, "window_border");
    apply_f32(&mut theme.window_border_width, &file.window_border_width);
    apply_color(&mut theme.steel_top, &file.steel_top, "steel_top");
    apply_color(&mut theme.steel_mid_a, &file.steel_mid_a, "steel_mid_a");
    apply_color(&mut theme.steel_mid_b, &file.steel_mid_b, "steel_mid_b");
    apply_color(&mut theme.steel_bottom, &file.steel_bottom, "steel_bottom");
    apply_color(&mut theme.steel_border, &file.steel_border, "steel_border");
    apply_f32(&mut theme.steel_corner_radius, &file.steel_corner_radius);
    apply_color(&mut theme.steel_shadow_color, &file.steel_shadow_color, "steel_shadow_color");
    apply_color(&mut theme.steel_text, &file.steel_text, "steel_text");
    apply_f32(&mut theme.steel_vertical_inset, &file.steel_vertical_inset);
    apply_color(&mut theme.right_panel_bg, &file.right_panel_bg, "right_panel_bg");
    apply_color(&mut theme.right_panel_border, &file.right_panel_border, "right_panel_border");
    apply_f32(&mut theme.right_panel_corner_radius, &file.right_panel_corner_radius);
    apply_color(&mut theme.widget_bg, &file.widget_bg, "widget_bg");
    apply_color(&mut theme.widget_border, &file.widget_border, "widget_border");
    apply_f32(&mut theme.widget_corner_radius, &file.widget_corner_radius);
    apply_f32(&mut theme.widget_scale, &file.widget_scale);
    apply_f32(&mut theme.widget_height, &file.widget_height);
    apply_f32(&mut theme.widget_height_tall, &file.widget_height_tall);
    if let Some(v) = file.widget_spacing {
        theme.widget_spacing = v;
    }
    apply_color(&mut theme.drawer_btn_bg, &file.drawer_btn_bg, "drawer_btn_bg");
    apply_color(&mut theme.drawer_btn_hover, &file.drawer_btn_hover, "drawer_btn_hover");
    apply_color(&mut theme.drawer_btn_active, &file.drawer_btn_active, "drawer_btn_active");
    apply_color(&mut theme.drawer_btn_border, &file.drawer_btn_border, "drawer_btn_border");
    apply_color(&mut theme.drawer_btn_text, &file.drawer_btn_text, "drawer_btn_text");
    apply_color(&mut theme.drawer_btn_text_hover, &file.drawer_btn_text_hover, "drawer_btn_text_hover");

    theme
}