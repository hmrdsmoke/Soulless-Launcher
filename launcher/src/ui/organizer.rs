// MIT License - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.

use cosmic::iced::{Element, Length, Theme};
use cosmic::iced::widget::{button, column, container, row, text};
use soulless_organizer::{OrganizerState, Message};
use crate::ui::theme;

pub fn organizer_banner<'a, M: 'static + Clone + Send>(
    state: &'a OrganizerState,
    map: impl Fn(Message) -> M + 'a + Clone,
) -> Option<Element<'a, M>>
{
    let suggestion = state.pending.first()?;
    let reason = text(&suggestion.suggestion.reason).size(12).color(cosmic::iced::Color::WHITE);
    let yes_btn = button(text("Move").size(12).color(cosmic::iced::Color::WHITE))
        .on_press({ let m = map.clone(); m(Message::ApproveSuggestion(0)) });
    let no_btn = button(text("Skip").size(12).color(cosmic::iced::Color::WHITE))
        .on_press(map(Message::DismissSuggestion(0)));
    let banner = container(
        column![reason, row![yes_btn, no_btn].spacing(8)].spacing(6).padding([8, 12])
    )
    .width(Length::Fill)
    .style(|_: &Theme| container::Style {
        background: Some(theme::WIDGET_BG.into()),
        border: cosmic::iced::Border {
            color: theme::WIDGET_BORDER,
            width: 1.0,
            radius: cosmic::iced::border::rounded(0).radius,
        },
        ..Default::default()
    });
    Some(banner.into())
}
