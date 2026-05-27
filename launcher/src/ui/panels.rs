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
use cosmic::iced::alignment::Vertical;

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
    let panels = launcher_panels(toolbox, right, bg_image_path);
    let monitors = monitor_grid(net, sys, hw, fps);

    let content = column![panels, monitors]
        .spacing(0)
        .height(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
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

fn launcher_panels<'a, M: 'static + Clone + Send>(
    drawers_content: Element<'a, M>,
    right_content: Element<'a, M>,
    bg_image_path: &Option<String>,
) -> Element<'a, M> {
    let steel = container(drawers_content)
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
        });

    let right_bg = if bg_image_path.is_some() {
        cosmic::iced::Color::from_rgba8(0, 0, 0, 0.0)
    } else {
        theme::RIGHT_PANEL_BG
    };
    let right_border = theme::RIGHT_PANEL_BORDER;

    let right = container(right_content)
        .width(Length::Fixed(RIGHT_PANEL_WIDTH))
        .height(Length::Fill)
        .style(move |_: &cosmic::iced::Theme| cosmic::iced::widget::container::Style {
            background: Some(right_bg.into()),
            border: cosmic::iced::Border {
                radius: cosmic::iced::border::rounded(theme::RIGHT_PANEL_CORNER_RADIUS).radius,
                color: right_border,
                width: 1.0,
            },
            icon_color: None,
            snap: false,
            ..Default::default()
        });

    container(
        row![
            steel,
            space::horizontal().width(Length::Fixed(12.0)),
            right,
        ]
        .spacing(0)
        .height(Length::Fill)
        .align_y(Vertical::Center)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn monitor_grid<'a, M: 'static + Clone + Send>(
    net: Element<'a, M>,
    sys: Element<'a, M>,
    hw: Element<'a, M>,
    fps: Element<'a, M>,
) -> Element<'a, M> {
    let net = container(net).style(widget_style);
    let sys = container(sys).style(widget_style);
    let hw  = container(hw).style(widget_style);
    let fps = container(fps).style(widget_style);

    column![
        row![net, sys].spacing(12),
        row![hw, fps].spacing(12),
    ]
    .spacing(12)
    .padding([0, 16, 16, 16])
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
