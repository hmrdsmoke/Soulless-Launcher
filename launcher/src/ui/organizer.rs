// MIT License - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

use cosmic::iced::{Element, Length};
use cosmic::iced::widget::{column, row, text};
use soulless_organizer::{OrganizerState, Message};
use crate::ui::theme;

pub fn organizer_banner<'a, M: 'static + Clone + Send>(
    state: &'a OrganizerState,
    map: impl Fn(Message) -> M + 'a + Clone,
) -> Option<Element<'a, M>>
{
    let suggestion = state.pending.first()?;

    let reason = text(&suggestion.suggestion.reason).size(11);

    let steel_active = cosmic::widget::button::Style {
        background: Some(cosmic::iced::Color::BLACK.into()),
        border_color: theme::STEEL_TOP,
        border_width: 1.0,
        border_radius: cosmic::iced::border::rounded(0).radius,
        text_color: Some(theme::STEEL_TOP),
        ..Default::default()
    };
    let steel_hovered = cosmic::widget::button::Style {
        background: Some(theme::STEEL_TOP.into()),
        border_color: theme::STEEL_TOP,
        border_width: 1.0,
        border_radius: cosmic::iced::border::rounded(0).radius,
        text_color: Some(cosmic::iced::Color::BLACK),
        ..Default::default()
    };

    let move_btn = cosmic::widget::button::custom(text("Move").size(11))
        .on_press({ let m = map.clone(); m(Message::ApproveSuggestion(0)) })
        .class(cosmic::theme::Button::Custom {
            active: Box::new(move |_, _| steel_active),
            hovered: Box::new(move |_, _| steel_hovered),
            pressed: Box::new(move |_, _| steel_hovered),
            disabled: Box::new(|_| cosmic::widget::button::Style::default()),
        });

    let steel_active2 = cosmic::widget::button::Style {
        background: Some(cosmic::iced::Color::BLACK.into()),
        border_color: theme::STEEL_TOP,
        border_width: 1.0,
        border_radius: cosmic::iced::border::rounded(0).radius,
        text_color: Some(theme::STEEL_TOP),
        ..Default::default()
    };
    let steel_hovered2 = cosmic::widget::button::Style {
        background: Some(theme::STEEL_TOP.into()),
        border_color: theme::STEEL_TOP,
        border_width: 1.0,
        border_radius: cosmic::iced::border::rounded(0).radius,
        text_color: Some(cosmic::iced::Color::BLACK),
        ..Default::default()
    };

    let skip_btn = cosmic::widget::button::custom(text("Skip").size(11))
        .on_press(map(Message::DismissSuggestion(0)))
        .class(cosmic::theme::Button::Custom {
            active: Box::new(move |_, _| steel_active2),
            hovered: Box::new(move |_, _| steel_hovered2),
            pressed: Box::new(move |_, _| steel_hovered2),
            disabled: Box::new(|_| cosmic::widget::button::Style::default()),
        });

    let banner = cosmic::iced::widget::container(
        column![reason, row![move_btn, skip_btn].spacing(8)].spacing(6).padding([8, 12])
    )
    .width(Length::Fill);

    Some(cosmic::iced::widget::Themer::new(None::<cosmic::Theme>, banner).into())
}
