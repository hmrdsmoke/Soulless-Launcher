// MIT License - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

// src/ui/panels.rs
// Composes the full launcher window layout.

use crate::ui::theme;
use crate::position::layout::{TOOLBOX_WIDTH, RIGHT_PANEL_WIDTH};
use cosmic::iced::{Element, Length};
use cosmic::iced::widget::{column, container, row, space};

/// Compose the full window — steel toolbox + right panel + monitor grid.
/// M is the top-level Message type from main.rs.
pub fn compose<'a, M: 'static + Clone + Send>(
    toolbox: Element<'a, M>,
    right: Element<'a, M>,
    net: Element<'a, M>,
    sys: Element<'a, M>,
    hw: Element<'a, M>,
    fps: Element<'a, M>,
    bg_image_path: &Option<String>,
) -> Element<'a, M> {
    let monitors = monitor_grid(net, sys, hw, fps);

    // Left column: steel toolbox + widgets below
    let left_col = column![
        launcher_steel(toolbox),
        monitors,
    ]
    .spacing(12)
    .width(Length::Fixed(crate::position::layout::TOOLBOX_WIDTH));

    // Right panel: full height
    let right_panel = right_content_panel(right, bg_image_path);

    let layout = row![
        left_col,
        space::horizontal().width(Length::Fixed(12.0)),
        right_panel,
    ]
    .spacing(0)
    .height(Length::Fill);

    container(layout)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(16)
        .clip(true)
        .style(|_: &cosmic::iced::Theme| cosmic::iced::widget::container::Style {
            background: Some(theme::WINDOW_BG.into()),
            border: cosmic::iced::Border {
                radius: cosmic::iced::border::rounded(theme::WINDOW_CORNER_RADIUS).radius,
                ..Default::default()
            },
            icon_color: None,
            snap: false,
            ..Default::default()
        })
        .into()
}

fn launcher_steel<'a, M: 'static + Clone + Send>(
    drawers_content: Element<'a, M>,
) -> Element<'a, M> {
    container(drawers_content)
        .width(Length::Fixed(TOOLBOX_WIDTH))
        .height(Length::Shrink)
        .padding([theme::STEEL_VERTICAL_INSET, 0.0, theme::STEEL_VERTICAL_INSET, 0.0])
        .style(|_: &cosmic::iced::Theme| cosmic::iced::widget::container::Style {
            background: Some(
                cosmic::iced::gradient::Linear::new(
                    cosmic::iced::Radians(std::f32::consts::PI * 0.55)
                )
                .add_stop(0.0, theme::STEEL_TOP)
                .add_stop(0.35, theme::STEEL_MID_A)
                .add_stop(0.65, theme::STEEL_MID_B)
                .add_stop(1.0, theme::STEEL_BOTTOM)
                .into()
            ),
            border: cosmic::iced::Border {
                radius: cosmic::iced::border::rounded(theme::STEEL_CORNER_RADIUS).radius,
                color: theme::STEEL_BORDER,
                width: 1.0,
            },
            shadow: cosmic::iced::Shadow {
                color: theme::STEEL_SHADOW_COLOR,
                offset: cosmic::iced::Vector::new(4.0, 4.0),
                blur_radius: 12.0,
            },
            text_color: Some(theme::STEEL_TEXT),
            icon_color: None,
            snap: false,
        })
    .into()
}

fn right_content_panel<'a, M: 'static + Clone + Send>(
    right_content: Element<'a, M>,
    bg_image_path: &Option<String>,
) -> Element<'a, M> {
    let right_border = theme::RIGHT_PANEL_BORDER;
    let width = crate::position::layout::RIGHT_PANEL_WIDTH;

    if let Some(path) = bg_image_path {
        // Pre-resize image to exact panel dimensions to prevent bleeding
        let handle = if let Some(rgba) = crate::config::load_background_rgba(
            path, width as u32, 900
        ) {
            cosmic::iced::widget::image::Handle::from_rgba(
                width as u32, 900u32, rgba
            )
        } else {
            cosmic::iced::widget::image::Handle::from_path(path.as_str())
        };

        let bg: Element<'a, M> = cosmic::iced::widget::image(handle)
            .width(Length::Fixed(width))
            .height(Length::Fill)
            .into();

        let overlay: Element<'a, M> = container(right_content)
            .width(Length::Fixed(width))
            .height(Length::Fill)
            .into();

        container(cosmic::iced::widget::stack([bg, overlay]))
            .width(Length::Fixed(width))
            .height(Length::Fixed(height))
            .clip(true)
            .style(move |_: &cosmic::iced::Theme| cosmic::iced::widget::container::Style {
                border: cosmic::iced::Border {
                    radius: cosmic::iced::border::rounded(theme::RIGHT_PANEL_CORNER_RADIUS).radius,
                    color: right_border,
                    width: 1.0,
                },
                icon_color: None,
                snap: false,
                ..Default::default()
            })
            .into()
    } else {
        container(right_content)
            .width(Length::Fixed(width))
            .height(Length::Fill)
            .style(move |_: &cosmic::iced::Theme| cosmic::iced::widget::container::Style {
                background: Some(theme::RIGHT_PANEL_BG.into()),
                border: cosmic::iced::Border {
                    radius: cosmic::iced::border::rounded(theme::RIGHT_PANEL_CORNER_RADIUS).radius,
                    color: right_border,
                    width: 1.0,
                },
                icon_color: None,
                snap: false,
                ..Default::default()
            })
            .into()
    }
}

fn monitor_grid<'a, M: 'static + Clone + Send>(
    net: Element<'a, M>,
    sys: Element<'a, M>,
    hw: Element<'a, M>,
    fps: Element<'a, M>,
) -> Element<'a, M> {
    // Each widget is 75% of half the window width (quarter less than full)
    // Widgets sit side by side in a single row under the steel panel
    // Each widget takes half the toolbox width with a small gap
    let widget_width = Length::Fixed(109.0);
    let widget_height = Length::Fixed(95.0);

    let net = container(net).width(widget_width).height(widget_height).style(widget_style);
    let sys = container(sys).width(widget_width).height(widget_height).style(widget_style);
    let hw  = container(hw).width(widget_width).height(widget_height).style(widget_style);
    let fps = container(fps).width(widget_width).height(widget_height).style(widget_style);

    column![
        row![net, sys].spacing(4),
        row![hw, fps].spacing(4),
    ]
    .spacing(4)
    .padding([8, 4, 4, 4])
    .into()
}

fn widget_style(_: &cosmic::iced::Theme) -> cosmic::iced::widget::container::Style {
    cosmic::iced::widget::container::Style {
        background: Some(theme::WIDGET_BG.into()),
        border: cosmic::iced::Border {
            radius: cosmic::iced::border::rounded(theme::WIDGET_CORNER_RADIUS).radius,
            color: theme::WIDGET_BORDER,
            width: 1.0,
        },
        icon_color: None,
        snap: false,
        ..Default::default()
    }
}
