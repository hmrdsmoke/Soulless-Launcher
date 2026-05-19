// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

use crate::search::Message as SearchMessage;
use crate::vault::{Vault, VaultLockState};

use cosmic::iced::widget::{
    column, container, mouse_area, row, scrollable, space, text,
    text_input,
};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Color, Element, Length, Theme};

pub fn view<'a>(vault: &'a Vault) -> Element<'a, SearchMessage> {
    match vault.lock_state {
        VaultLockState::Uninitialized => setup_view(vault),
        VaultLockState::Locked => unlock_view(vault),
        VaultLockState::Unlocked => files_view(vault),
    }
}

// ── Setup view (first launch) ─────────────────────────────────────────────────

fn setup_view<'a>(vault: &'a Vault) -> Element<'a, SearchMessage> {
    let title = text("🔒 Create Vault Password").size(24);

    let subtitle = text(
        "This password encrypts everything in your vault.\n\
         If you forget it, your files cannot be recovered."
    )
    .size(13);

    let password_field = text_input("Password (min 8 chars)", &vault.password_input)
        .on_input(SearchMessage::VaultPasswordChanged)
        .secure(true)
        .padding(12)
        .size(15);

    let confirm_field = text_input("Confirm password", &vault.confirm_input)
        .on_input(SearchMessage::VaultConfirmChanged)
        .secure(true)
        .padding(12)
        .size(15);

    let create_btn = mouse_area(
        container(
            text("Create Vault").size(15)
        )
        .padding([10, 24])
        .style(|_: &Theme| container::Style {
            background: Some(Color::from_rgb8(60, 60, 180).into()),
            border: cosmic::iced::border::rounded(8),
            ..Default::default()
        })
    )
    .on_press(SearchMessage::VaultSetupConfirm);

    let mut col = column![
        title,
        space::vertical().height(Length::Fixed(8.0)),
        subtitle,
        space::vertical().height(Length::Fixed(24.0)),
        password_field,
        space::vertical().height(Length::Fixed(12.0)),
        confirm_field,
        space::vertical().height(Length::Fixed(20.0)),
        create_btn,
    ]
    .spacing(0)
    .align_x(Horizontal::Center)
    .width(Length::Fixed(340.0));

    if let Some(err) = &vault.error {
        col = col.push(space::vertical().height(Length::Fixed(12.0)));
        col = col.push(
            text(err.as_str()).size(13).color(Color::from_rgb8(220, 80, 80))
        );
    }

    container(col)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(vault_bg)
        .into()
}

// ── Unlock view ───────────────────────────────────────────────────────────────

fn unlock_view<'a>(vault: &'a Vault) -> Element<'a, SearchMessage> {
    let title = text("🔒 Vault").size(24);

    let password_field = text_input("Enter password", &vault.password_input)
        .on_input(SearchMessage::VaultPasswordChanged)
        .on_submit(SearchMessage::VaultUnlock)
        .secure(true)
        .padding(12)
        .size(15);

    let unlock_btn = mouse_area(
        container(text("Unlock").size(15))
            .padding([10, 32])
            .style(|_: &Theme| container::Style {
                background: Some(Color::from_rgb8(60, 60, 180).into()),
                border: cosmic::iced::border::rounded(8),
                ..Default::default()
            })
    )
    .on_press(SearchMessage::VaultUnlock);

    let mut col = column![
        title,
        space::vertical().height(Length::Fixed(24.0)),
        password_field,
        space::vertical().height(Length::Fixed(16.0)),
        unlock_btn,
    ]
    .spacing(0)
    .align_x(Horizontal::Center)
    .width(Length::Fixed(300.0));

    if let Some(err) = &vault.error {
        col = col.push(space::vertical().height(Length::Fixed(12.0)));
        col = col.push(
            text(err.as_str()).size(13).color(Color::from_rgb8(220, 80, 80))
        );
    }

    container(col)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(vault_bg)
        .into()
}

// ── Files view (unlocked) ─────────────────────────────────────────────────────

fn files_view<'a>(vault: &'a Vault) -> Element<'a, SearchMessage> {
    let header = row![
        text("🔓 Vault").size(22),
        space::horizontal().width(Length::Fill),
        mouse_area(
            container(text("🔒 Lock").size(13))
                .padding([6, 14])
                .style(|_: &Theme| container::Style {
                    background: Some(Color::from_rgb8(80, 40, 40).into()),
                    border: cosmic::iced::border::rounded(6),
                    ..Default::default()
                })
        )
        .on_press(SearchMessage::VaultLock),
    ]
    .align_y(Vertical::Center)
    .padding([0, 0, 16, 0]);

    let status_bar: Element<'a, SearchMessage> =
        if let Some(err) = &vault.error {
            text(err.as_str())
                .size(12)
                .color(Color::from_rgb8(220, 80, 80))
                .into()
        } else if let Some(status) = &vault.status {
            text(status.as_str())
                .size(12)
                .color(Color::from_rgb8(80, 200, 120))
                .into()
        } else {
            space::vertical().height(Length::Fixed(0.0)).into()
        };

    let drop_hint = container(
        text("Drag files here to add them to your vault")
            .size(13)
            .center()
    )
    .width(Length::Fill)
    .padding(16)
    .style(|_: &Theme| container::Style {
        background: Some(Color::from_rgba8(255, 255, 255, 0.03).into()),
        border: cosmic::iced::border::rounded(8),
        ..Default::default()
    });

    let file_list: Element<'a, SearchMessage> = if vault.entries.is_empty() {
        container(
            column![
                space::vertical().height(Length::Fixed(32.0)),
                text("Your vault is empty.").size(15).center(),
                space::vertical().height(Length::Fixed(8.0)),
                text("Drag files in to get started.").size(12).center(),
            ]
            .align_x(Horizontal::Center)
        )
        .width(Length::Fill)
        .center_x(Length::Fill)
        .into()
    } else {
        let rows = vault.entries.iter().fold(
            column!().spacing(4),
            |col, entry| col.push(file_row(entry)),
        );

        scrollable(container(rows).padding([0, 8, 0, 0]))
            .height(Length::Fill)
            .into()
    };

    container(
        column![
            header,
            status_bar,
            space::vertical().height(Length::Fixed(8.0)),
            drop_hint,
            space::vertical().height(Length::Fixed(12.0)),
            file_list,
        ]
        .spacing(0)
        .width(Length::Fill)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(20)
    .style(vault_bg)
    .into()
}

// ── Individual file row ───────────────────────────────────────────────────────

fn file_row<'a>(
    entry: &'a crate::vault::VaultEntry,
) -> Element<'a, SearchMessage> {
    let icon = file_icon(&entry.meta.mime_type);
    let name = text(entry.meta.original_name.as_str()).size(14);
    let size = text(format_size(entry.meta.size)).size(11);

    let entry_id = entry.id.clone();
    let entry_id_remove = entry.id.clone();

    mouse_area(
        container(
            row![
                text(icon).size(20),
                space::horizontal().width(Length::Fixed(12.0)),
                column![name, size].spacing(2),
                space::horizontal().width(Length::Fill),
            ]
            .align_y(Vertical::Center)
            .padding([8, 12])
        )
        .width(Length::Fill)
        .style(|_: &Theme| container::Style {
            background: Some(Color::from_rgba8(255, 255, 255, 0.04).into()),
            border: cosmic::iced::border::rounded(6),
            ..Default::default()
        })
    )
    .on_press(SearchMessage::VaultOpenFile(entry_id))
    .on_right_press(SearchMessage::VaultRemoveFile(entry_id_remove))
    .into()
}

// ── Style helpers ─────────────────────────────────────────────────────────────

fn vault_bg(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgb8(18, 18, 28).into()),
        border: cosmic::iced::border::rounded(12),
        ..Default::default()
    }
}

fn file_icon(mime: &str) -> &'static str {
    if mime.starts_with("video/") {
        "🎬"
    } else if mime.starts_with("image/") {
        "🖼"
    } else if mime.starts_with("audio/") {
        "🎵"
    } else if mime.contains("pdf") {
        "📄"
    } else if mime.contains("zip")
        || mime.contains("tar")
        || mime.contains("gz")
    {
        "📦"
    } else {
        "📁"
    }
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

// === DONE ===
// Replaced .password() with .secure(true) — correct method for this iced version :: done
// from_rgba8 fixed — takes u8 0-255 not f32 0.0-1.0 :: done
// All three views preserved: setup, unlock, files :: done
// Error in red, status in green :: done
// Click to open, right-click to remove :: done

// === DONE ===
// Setup view: create password + confirm :: done
// Unlock view: enter password, submit on Enter :: done
// Files view: header, lock button, drop hint, file list :: done
// File row: icon by mime type, name, size, click to open, right-click to remove :: done
// Error shown in red, status shown in green :: done
// vault_bg style applied to all three views :: done