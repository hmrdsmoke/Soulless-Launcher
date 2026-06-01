// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

use crate::search::Message as SearchMessage;
use super::{Vault, VaultLockState};

use cosmic::iced::widget::{
    column, container, mouse_area, row, scrollable, space, text,
    text_input,
};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Color, Element, Length, Theme};

// dnd_destination lives in cosmic::widget and uses cosmic::Theme.
// Themer bridges a cosmic::Theme subtree into our cosmic::iced::Theme tree:
//   Themer::new(None, cosmic_element) → cosmic::iced::Element
// None = inherit theme from parent, no override needed.
use cosmic::widget::dnd_destination;
use cosmic::iced::widget::Themer;

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
         If you forget it, your files cannot be recovered.",
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
        container(text("Create Vault").size(15))
            .padding([10, 24])
            .style(|_: &Theme| container::Style {
                background: Some(Color::from_rgb8(60, 60, 180).into()),
                border: cosmic::iced::border::rounded(8),
                ..Default::default()
            }),
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
            text(err.as_str())
                .size(13)
                .color(Color::from_rgb8(220, 80, 80)),
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
            }),
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
            text(err.as_str())
                .size(13)
                .color(Color::from_rgb8(220, 80, 80)),
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

    // ── Drop zone ─────────────────────────────────────────────────────────────
    let (drop_bg, drop_border_color) = if vault.drag_hover {
        (
            Color::from_rgba8(80, 120, 255, 0.12),
            Color::from_rgb8(80, 120, 255),
        )
    } else {
        (
            Color::from_rgba8(255, 255, 255, 0.03),
            Color::from_rgba8(255, 255, 255, 0.08),
        )
    };

    let drop_label = if vault.drag_hover {
        "Drop to add to vault"
    } else {
        "Drag files here to add them to your vault"
    };

    // Step 1: build inner visual as cosmic::Element
    let drop_inner: cosmic::Element<'_, SearchMessage> =
        cosmic::widget::container(
            cosmic::widget::text(drop_label).size(13),
        )
        .width(Length::Fill)
        .padding(16)
        .into();

    // Step 2: wrap with dnd_destination — still cosmic::Element.
    // This registers the widget bounds with the Wayland compositor as a
    // drop target, which is what makes Event::Dnd events actually arrive.
    let drop_dest: cosmic::Element<'_, SearchMessage> = dnd_destination(
        drop_inner,
        vec![std::borrow::Cow::Borrowed("text/uri-list")],
    )
    .on_enter(|_x, _y, _mimes| SearchMessage::VaultDragHover(true))
    .on_leave(|| SearchMessage::VaultDragHover(false))
    .on_finish(|_mime, data, _action, _x, _y| {
        let payload = String::from_utf8_lossy(&data);
        let paths = payload
            .lines()
            .map(str::trim)
            .filter(|l| l.starts_with("file://"))
            .filter_map(|l| {
                let raw = l.trim_start_matches("file://");
                let decoded = crate::utils::percent_decode_uri(raw);
                let p = std::path::PathBuf::from(decoded);
                if p.exists() { Some(p) } else { None }
            })
            .collect::<Vec<_>>();
        SearchMessage::VaultFilesDropped(paths)
    })
    .into();

    // Step 3: Themer bridges cosmic::Theme → cosmic::iced::Theme.
    // None = inherit theme from parent (no visual override).
    // The outer iced container applies the hover styling on top.
    let drop_zone: Element<'_, SearchMessage> =
        container(
            Themer::new(None::<cosmic::Theme>, drop_dest),
        )
        .width(Length::Fill)
        .style(move |_: &Theme| container::Style {
            background: Some(drop_bg.into()),
            border: cosmic::iced::Border {
                color: drop_border_color,
                width: if vault.drag_hover { 1.5 } else { 1.0 },
                radius: cosmic::iced::border::rounded(8).radius,
            },
            ..Default::default()
        })
        .into();

    // ── File list ─────────────────────────────────────────────────────────────

    let file_list: Element<'a, SearchMessage> = if vault.entries.is_empty() {
        container(
            column![
                space::vertical().height(Length::Fixed(32.0)),
                text("Your vault is empty.").size(15).center(),
                space::vertical().height(Length::Fixed(8.0)),
                text("Drag files in to get started.").size(12).center(),
            ]
            .align_x(Horizontal::Center),
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


    let main_col: Element<'a, SearchMessage> = container(
        column![
            header,
            status_bar,
            space::vertical().height(Length::Fixed(8.0)),
            drop_zone,
            space::vertical().height(Length::Fixed(12.0)),
            file_list,
        ]
        .spacing(0)
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(20)
    .style(vault_bg)
    .into();

    if let Some(ref entry_id) = vault.context_menu_entry {
        let entry_name = vault.entries.iter()
            .find(|e| &e.id == entry_id)
            .map(|e| e.meta.original_name.as_str())
            .unwrap_or("File");
        let menu = vault_context_menu(entry_id.clone(), entry_name);
        cosmic::iced::widget::stack([main_col, menu]).into()
    } else {
        main_col
    }
}

// ── Individual file row ───────────────────────────────────────────────────────

fn file_row<'a>(
    entry: &'a super::VaultEntry,
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
            .padding([8, 12]),
        )
        .width(Length::Fill)
        .style(|_: &Theme| container::Style {
            background: Some(Color::from_rgba8(255, 255, 255, 0.04).into()),
            border: cosmic::iced::border::rounded(6),
            ..Default::default()
        }),
    )
    .on_press(SearchMessage::VaultOpenFile(entry_id))
    .on_right_press(SearchMessage::VaultOpenFileMenu(entry_id_remove))
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

// ── Vault file context menu ───────────────────────────────────────────────────

fn vault_context_menu<'a>(entry_id: String, name: &'a str) -> Element<'a, SearchMessage> {
    let id_open = entry_id.clone();
    let id_export = entry_id.clone();
    let id_remove = entry_id.clone();

    let open_btn = mouse_area(
        container(text("📂 Open").size(13))
            .padding([8, 16])
            .width(Length::Fill)
            .style(|_: &Theme| container::Style {
                background: Some(Color::from_rgba8(255,255,255,0.05).into()),
                ..Default::default()
            })
    ).on_press(SearchMessage::VaultOpenFile(id_open));

    let export_btn = mouse_area(
        container(text("💾 Export to Downloads").size(13))
            .padding([8, 16])
            .width(Length::Fill)
            .style(|_: &Theme| container::Style {
                background: Some(Color::from_rgba8(255,255,255,0.05).into()),
                ..Default::default()
            })
    ).on_press(SearchMessage::VaultExportFile(id_export));

    let remove_btn = mouse_area(
        container(text("🗑 Remove from vault").size(13))
            .padding([8, 16])
            .width(Length::Fill)
            .style(|_: &Theme| container::Style {
                background: Some(Color::from_rgba8(180,40,40,0.15).into()),
                ..Default::default()
            })
    ).on_press(SearchMessage::VaultRemoveFile(id_remove));

    let menu = container(
        column![
            text(name).size(11),
            open_btn,
            export_btn,
            remove_btn,
        ]
        .spacing(2)
        .width(Length::Fixed(200.0)),
    )
    .padding(8)
    .style(|_: &Theme| container::Style {
        background: Some(Color::from_rgb8(28, 28, 38).into()),
        border: cosmic::iced::Border {
            color: Color::from_rgba8(255,255,255,0.15),
            width: 1.0,
            radius: cosmic::iced::border::rounded(8).radius,
        },
        ..Default::default()
    });

    mouse_area(
        container(menu)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
    )
    .on_press(SearchMessage::VaultCloseContextMenu)
    .into()
}
