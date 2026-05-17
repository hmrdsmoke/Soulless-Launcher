// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

use cosmic::iced::widget::{button, column, container, space, text};
use cosmic::iced::{Color, Element, Length};
use crate::search::Message as SearchMessage;

pub fn view() -> Element<'static, SearchMessage> {
    container(
        column![
            text("🔒 Secure Vault").size(28),
            space::vertical().height(Length::Fixed(16.0)),
            text("Your encrypted files and apps go here.").size(16),
            space::vertical().height(Length::Fixed(24.0)),
            button(text("Open Vault (Coming Soon)"))
                .width(Length::Fixed(220.0))
                .height(Length::Fixed(48.0))
                .style(|_theme, _status| button::Style {
                    background: Some(Color::from_rgb8(60, 60, 80).into()),
                    border: cosmic::iced::border::rounded(8),
                    ..Default::default()
                })
                .on_press(SearchMessage::VaultClicked)
        ]
        .align_x(cosmic::iced::alignment::Horizontal::Center)
        .padding(40),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_| container::Style {
        background: Some(Color::from_rgb8(25, 25, 35).into()),
        border: cosmic::iced::border::rounded(12),
        ..Default::default()
    })
    .into()
}

// === DONE ===
// Fixed: bare `iced` imports replaced with `cosmic::iced` :: done
// Fixed: iced::alignment::Horizontal → cosmic::iced::alignment::Horizontal :: done
// Fixed: iced::border::rounded → cosmic::iced::border::rounded :: done
// Vault is placeholder — encryption logic to be added :: MRV
// Ready for future integration with age or other crypto :: MRV
// === YOUR ORIGINAL COMMENTS (preserved exactly) ===
// Vault is placeholder for now - encryption logic to be added :: MRV
// Ready for future integration with age or other crypto :: MRV
// === DONE ===
// Basic vault UI structure ready for expansion