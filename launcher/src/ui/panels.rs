// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/ui/panels.rs
// Composes the full launcher window layout.

use crate::ui::theme;
use crate::position::layout::TOOLBOX_WIDTH;
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
    bg_handle: Option<cosmic::iced::widget::image::Handle>,
) -> Element<'a, M> {
    let monitors = monitor_grid(net, sys, hw, fps);

    // Left column: steel toolbox + widgets below
    let left_col = column![
        launcher_steel(toolbox),
        monitors,
    ]
    .spacing(12)
    .width(Length::Fixed(TOOLBOX_WIDTH));

    // Right panel: full height
    let right_panel = right_content_panel(right, bg_handle);

    let layout = row![
        left_col,
        space::horizontal().width(Length::Fixed(crate::position::layout::PANEL_SPACING)),
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
            background: Some(theme::get().window_bg.into()),
            border: cosmic::iced::Border {
                radius: cosmic::iced::border::rounded(theme::get().window_corner_radius).radius,
                color: theme::get().window_border,
                width: theme::get().window_border_width,
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
        .padding([theme::get().steel_vertical_inset, 0.0, theme::get().steel_vertical_inset, 0.0])
        .style(|_: &cosmic::iced::Theme| cosmic::iced::widget::container::Style {
            background: Some(
                cosmic::iced::gradient::Linear::new(
                    cosmic::iced::Radians(std::f32::consts::PI * 0.55)
                )
                .add_stop(0.0, theme::get().steel_top)
                .add_stop(0.35, theme::get().steel_mid_a)
                .add_stop(0.65, theme::get().steel_mid_b)
                .add_stop(1.0, theme::get().steel_bottom)
                .into()
            ),
            border: cosmic::iced::Border {
                radius: cosmic::iced::border::rounded(theme::get().steel_corner_radius).radius,
                color: theme::get().steel_border,
                width: 1.0,
            },
            shadow: cosmic::iced::Shadow {
                color: theme::get().steel_shadow_color,
                offset: cosmic::iced::Vector::new(4.0, 4.0),
                blur_radius: 12.0,
            },
            text_color: Some(theme::get().steel_text),
            icon_color: None,
            snap: false,
        })
    .into()
}

fn right_content_panel<'a, M: 'static + Clone + Send>(
    right_content: Element<'a, M>,
    bg_handle: Option<cosmic::iced::widget::image::Handle>,
) -> Element<'a, M> {
    let right_border = theme::get().right_panel_border;
    let width = crate::position::layout::RIGHT_PANEL_WIDTH;

    if let Some(handle) = bg_handle {
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
            .height(Length::Fill)
            .clip(true)
            .style(move |_: &cosmic::iced::Theme| cosmic::iced::widget::container::Style {
                border: cosmic::iced::Border {
                    radius: cosmic::iced::border::rounded(theme::get().right_panel_corner_radius).radius,
                    color: right_border,
                    width: 0.0,
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
                background: Some(theme::get().right_panel_bg.into()),
                border: cosmic::iced::Border {
                    radius: cosmic::iced::border::rounded(theme::get().right_panel_corner_radius).radius,
                    color: right_border,
                    width: 0.0,
                },
                icon_color: None,
                snap: false,
                ..Default::default()
            })
            .into()
    }
}

// TODO: move to ui/widgets.rs — see refactor plan
fn monitor_grid<'a, M: 'static + Clone + Send>(
    net: Element<'a, M>,
    sys: Element<'a, M>,
    hw: Element<'a, M>,
    fps: Element<'a, M>,
) -> Element<'a, M> {
    // Each widget is 75% of half the window width (quarter less than full)
    // Widgets sit side by side in a single row under the steel panel
    // Each widget takes half the toolbox width with a small gap
    let widget_height = Length::Fixed(theme::get().widget_height);

    let net = container(net).width(Length::Fill).height(widget_height).style(widget_style);
    let sys = container(sys).width(Length::Fill).height(widget_height).style(widget_style);
    let hw  = container(hw).width(Length::Fill).height(widget_height).style(widget_style);
    let fps = container(fps).width(Length::Fill).height(widget_height).style(widget_style);

    column![
        row![net, sys].spacing(theme::get().widget_spacing),
        row![hw, fps].spacing(theme::get().widget_spacing),
    ]
    .spacing(4)
    .padding([8, 0, 4, 0])
    .into()
}

// TODO: move to ui/widgets.rs — see refactor plan
fn widget_style(_: &cosmic::iced::Theme) -> cosmic::iced::widget::container::Style {
    cosmic::iced::widget::container::Style {
        background: Some(theme::get().widget_bg.into()),
        border: cosmic::iced::Border {
            radius: cosmic::iced::border::rounded(theme::get().widget_corner_radius).radius,
            color: theme::get().widget_border,
            width: 1.0,
        },
        icon_color: None,
        snap: false,
        ..Default::default()
    }
}
