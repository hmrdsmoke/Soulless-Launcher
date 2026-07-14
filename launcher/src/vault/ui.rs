// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.
// launcher/src/vault/ui.rs
// Vault UI - unlock prompt and hidden-app grid views.

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

pub fn view<'a>(vault: &'a Vault, cursor_pos: cosmic::iced::Point) -> Element<'a, SearchMessage> {
    match vault.lock_state {
        VaultLockState::Uninitialized => setup_view(vault),
        VaultLockState::Locked => unlock_view(vault),
        VaultLockState::Unlocked => files_view(vault, cursor_pos),
        VaultLockState::NeedsUpgrade => upgrade_view(vault),
    }
}

// ── Setup view (first launch) ─────────────────────────────────────────────────

fn setup_view<'a>(vault: &'a Vault) -> Element<'a, SearchMessage> {
    let title = text("🔒 Create Vault Password").size(24);

    let banner = text(
        "☠ THIS IS A DEAD MAN'S VAULT\n\
         Do not put anything in here that you would like to recover.\n\
         The whole point is to NOT recover.",
    )
    .size(12)
    .color(Color::from_rgb8(220, 120, 90));

    let subtitle = text(
        "This password encrypts everything in your vault.\n\
         If you forget it, your files cannot be recovered.",
    )
    .size(13)
    .align_x(cosmic::iced::alignment::Horizontal::Center);

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

    // Black field outlined in steel; flips silver-with-ink on hover — same
    // invert language as the menus. (Button::Custom gives per-status styling;
    // the old static container couldn't hover.)
    let create_btn_inner: cosmic::Element<'_, SearchMessage> =
        cosmic::widget::button::custom(cosmic::widget::text("Create Vault").size(15))
            .padding([10, 24])
            .on_press(SearchMessage::VaultSetupConfirm)
            .class(cosmic::theme::Button::Custom {
                active: Box::new(|_selected, _theme| {
                    let t = crate::ui::theme::get();
                    cosmic::widget::button::Style {
                        background: Some(t.window_bg.into()),
                        border_width: 1.0,
                        border_color: t.text_steel,
                        text_color: Some(t.text_steel),
                        border_radius: cosmic::iced::border::rounded(0).radius,
                        ..Default::default()
                    }
                }),
                hovered: Box::new(|_selected, _theme| {
                    let t = crate::ui::theme::get();
                    cosmic::widget::button::Style {
                        background: Some(t.drawer_btn_hover.into()),
                        border_width: 1.0,
                        border_color: t.text_ink,
                        text_color: Some(t.text_ink),
                        border_radius: cosmic::iced::border::rounded(0).radius,
                        ..Default::default()
                    }
                }),
                pressed: Box::new(|_selected, _theme| {
                    let t = crate::ui::theme::get();
                    cosmic::widget::button::Style {
                        background: Some(t.drawer_btn_active.into()),
                        border_width: 1.0,
                        border_color: t.text_ink,
                        text_color: Some(t.text_ink),
                        border_radius: cosmic::iced::border::rounded(0).radius,
                        ..Default::default()
                    }
                }),
                disabled: Box::new(|_theme| cosmic::widget::button::Style::default()),
            })
            .into();
    let create_btn = cosmic::iced::widget::Themer::new(None::<cosmic::Theme>, create_btn_inner);

    let mut col = column![
        title,
        space::vertical().height(Length::Fixed(10.0)),
        banner,
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

// ── Upgrade view (old vault format detected) ──────────────────────────────────

fn upgrade_view<'a>(vault: &'a Vault) -> Element<'a, SearchMessage> {
    let title = text("⚠ Vault Security Upgrade").size(24);

    let body = text(
        "Your vault uses an older encryption format. To upgrade to the new \
         hardened format, the vault must be reset.\n\n\
         BEFORE YOU CONTINUE:\n\
         1. A backup of your current vault will be made automatically.\n\
         2. After upgrading, your old files will NOT carry over.\n\
         3. Re-add your files once the new vault is created.\n\n\
         Your backup will be saved next to the vault directory. To keep your \
         files, export anything important from the old vault first (open it in \
         the previous app version), then return here.",
    )
    .size(13);

    let upgrade_btn = mouse_area(
        container(text("Back up & Upgrade Vault").size(15))
            .padding([10, 24])
            .style(|_: &Theme| container::Style {
                background: Some(Color::from_rgb8(150, 90, 30).into()),
                border: cosmic::iced::border::rounded(0),
                ..Default::default()
            }),
    )
    .on_press(SearchMessage::VaultConfirmUpgrade);

    let mut col = column![
        title,
        space::vertical().height(Length::Fixed(16.0)),
        body,
        space::vertical().height(Length::Fixed(24.0)),
        upgrade_btn,
    ]
    .spacing(0)
    .align_x(Horizontal::Center)
    .width(Length::Fixed(420.0));

    if let Some(err) = &vault.error {
        col = col.push(space::vertical().height(Length::Fixed(12.0)));
        col = col.push(
            text(err.as_str())
                .size(13)
                .color(Color::from_rgb8(220, 80, 80)),
        );
    }
    if let Some(status) = &vault.status {
        col = col.push(space::vertical().height(Length::Fixed(12.0)));
        col = col.push(
            text(status.as_str())
                .size(12)
                .color(Color::from_rgb8(80, 200, 120)),
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
        .id(cosmic::widget::Id::new("vault-password"))
        .on_input(SearchMessage::VaultPasswordChanged)
        .on_submit(SearchMessage::VaultUnlock)
        .secure(true)
        .padding(12)
        .size(15);

    // Same treatment as create_btn: black, steel outline, silver-flip hover.
    let unlock_btn_inner: cosmic::Element<'_, SearchMessage> =
        cosmic::widget::button::custom(cosmic::widget::text("Unlock").size(15))
            .padding([10, 32])
            .on_press(SearchMessage::VaultUnlock)
            .class(cosmic::theme::Button::Custom {
                active: Box::new(|_selected, _theme| {
                    let t = crate::ui::theme::get();
                    cosmic::widget::button::Style {
                        background: Some(t.window_bg.into()),
                        border_width: 1.0,
                        border_color: t.text_steel,
                        text_color: Some(t.text_steel),
                        border_radius: cosmic::iced::border::rounded(0).radius,
                        ..Default::default()
                    }
                }),
                hovered: Box::new(|_selected, _theme| {
                    let t = crate::ui::theme::get();
                    cosmic::widget::button::Style {
                        background: Some(t.drawer_btn_hover.into()),
                        border_width: 1.0,
                        border_color: t.text_ink,
                        text_color: Some(t.text_ink),
                        border_radius: cosmic::iced::border::rounded(0).radius,
                        ..Default::default()
                    }
                }),
                pressed: Box::new(|_selected, _theme| {
                    let t = crate::ui::theme::get();
                    cosmic::widget::button::Style {
                        background: Some(t.drawer_btn_active.into()),
                        border_width: 1.0,
                        border_color: t.text_ink,
                        text_color: Some(t.text_ink),
                        border_radius: cosmic::iced::border::rounded(0).radius,
                        ..Default::default()
                    }
                }),
                disabled: Box::new(|_theme| cosmic::widget::button::Style::default()),
            })
            .into();
    let unlock_btn = cosmic::iced::widget::Themer::new(None::<cosmic::Theme>, unlock_btn_inner);

    // Dead man's switch reachable from the lock screen. No password recovery
    // exists by design; the only "forgot password" action is to DESTROY the
    // vault and start fresh. Deliberately styled as destructive and placed
    // well below Unlock so it can't be hit by a stray click.
    let destroy_btn = mouse_area(
        container(
            text("Forgot password?  →  Destroy vault")
                .size(12),
        )
        .padding([8, 18])
        .style(|_: &Theme| container::Style {
            background: Some(Color::from_rgb8(110, 30, 30).into()),
            border: cosmic::iced::Border {
                color: Color::from_rgb8(200, 60, 60),
                width: 1.0,
                radius: cosmic::iced::border::rounded(0).radius,
            },
            ..Default::default()
        }),
    )
    .on_press(SearchMessage::VaultForgetDestroy);

    let mut col = column![
        title,
        space::vertical().height(Length::Fixed(24.0)),
        password_field,
        space::vertical().height(Length::Fixed(16.0)),
        unlock_btn,
        space::vertical().height(Length::Fixed(40.0)),
        // align_x: iced left-aligns wrapped/multi-line text by default, so the
        // column centered the BLOCK while its lines stayed ragged inside it.
        text("This vault cannot be recovered. There is no password reset —\n\
              only permanent destruction.")
            .size(10)
            .align_x(cosmic::iced::alignment::Horizontal::Center)
            .color(Color::from_rgb8(150, 150, 160)),
        space::vertical().height(Length::Fixed(8.0)),
        destroy_btn,
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

fn files_view<'a>(vault: &'a Vault, cursor_pos: cosmic::iced::Point) -> Element<'a, SearchMessage> {
    let header = row![
        text("🔓 Vault").size(22),
        space::horizontal().width(Length::Fill),
        mouse_area(
            container(text("🔒 Lock").size(13))
                .padding([6, 14])
                .style(|_: &Theme| container::Style {
                    background: Some(Color::from_rgb8(80, 40, 40).into()),
                    border: cosmic::iced::border::rounded(0),
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
                radius: cosmic::iced::border::rounded(0).radius,
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


    let hidden_grid: Element<'a, SearchMessage> = hidden_apps_grid(vault)
        .unwrap_or_else(|| space::vertical().height(Length::Fixed(0.0)).into());
    let main_col: Element<'a, SearchMessage> = container(
        column![
            header,
            status_bar,
            space::vertical().height(Length::Fixed(8.0)),
            drop_zone,
            space::vertical().height(Length::Fixed(12.0)),
            hidden_grid,
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
        let menu = vault_context_menu(entry_id.clone(), entry_name, cursor_pos);
        cosmic::iced::widget::stack([main_col, menu]).into()
    } else if let Some(id) = &vault.hidden_context_menu {
        if let Some(app) = vault.hidden_apps.iter().find(|a| &a.id == id) {
            let menu = hidden_app_menu(app, cursor_pos);
            cosmic::iced::widget::stack([main_col, menu]).into()
        } else {
            main_col
        }
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
            border: cosmic::iced::border::rounded(0),
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
        border: cosmic::iced::border::rounded(0),
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

// ── Popup-surface menu renderers ──────────────────────────────────────────────
// These render the bare menu box (no cursor-inset positioning): the compositor
// positions the popup surface, so the menu just needs its content + styling.

/// Vault file-entry menu, rendered into its own popup surface.
pub fn vault_menu_popup<'a>(entry_id: &str, name: &'a str) -> Element<'a, SearchMessage> {
    let id_open = entry_id.to_string();
    let id_export = entry_id.to_string();
    let id_remove = entry_id.to_string();

    let btn = |label: &'static str, msg: SearchMessage, danger: bool| -> Element<'a, SearchMessage> {
        let bg = if danger {
            Color::from_rgba8(180, 40, 40, 0.15)
        } else {
            Color::from_rgba8(255, 255, 255, 0.05)
        };
        mouse_area(
            container(text(label).size(13))
                .padding([8, 16])
                .width(Length::Fill)
                .style(move |_: &Theme| container::Style {
                    background: Some(bg.into()),
                    ..Default::default()
                }),
        )
        .on_press(msg)
        .into()
    };

    container(
        column![
            text(name).size(11),
            btn("📂 Open", SearchMessage::VaultOpenFile(id_open), false),
            btn("💾 Export to Downloads", SearchMessage::VaultExportFile(id_export), false),
            btn("🗑 Remove from vault", SearchMessage::VaultRemoveFile(id_remove), true),
        ]
        .spacing(2)
        .width(Length::Fixed(200.0)),
    )
    .padding(8)
    .style(|_: &Theme| container::Style {
        background: Some(Color::from_rgb8(28, 28, 38).into()),
        border: cosmic::iced::Border {
            color: Color::from_rgba8(255, 255, 255, 0.15),
            width: 1.0,
            radius: cosmic::iced::border::rounded(0).radius,
        },
        ..Default::default()
    })
    .into()
}

/// Vault hidden-app menu, rendered into its own popup surface.
pub fn vault_hidden_menu_popup<'a>(app_id: &str) -> Element<'a, SearchMessage> {
    let id_launch = app_id.to_string();
    let id_remove = app_id.to_string();

    let item = |label: &'static str, msg: SearchMessage| -> Element<'a, SearchMessage> {
        mouse_area(
            container(text(label).size(13))
                .padding([6, 10])
                .width(Length::Fill),
        )
        .on_press(msg)
        .into()
    };

    container(
        column![
            item("↗ Launch", SearchMessage::LaunchHiddenApp(id_launch)),
            item("📤 Remove from vault", SearchMessage::RemoveFromVault(id_remove)),
        ]
        .spacing(2),
    )
    .padding(6)
    .width(Length::Fixed(180.0))
    .style(|_: &Theme| container::Style {
        background: Some(Color::from_rgb8(30, 30, 38).into()),
        border: cosmic::iced::Border {
            radius: 0.0.into(),
            width: 1.0,
            color: Color::from_rgb8(90, 90, 110),
        },
        ..Default::default()
    })
    .into()
}

// ── Vault file context menu ───────────────────────────────────────────────────

fn vault_context_menu<'a>(entry_id: String, name: &'a str, cursor_pos: cosmic::iced::Point) -> Element<'a, SearchMessage> {
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
            radius: cosmic::iced::border::rounded(0).radius,
        },
        ..Default::default()
    });

    // The vault view renders inside the right panel, whose top-left is inset
    // from the surface origin by outer padding (16) + toolbox (220) + panel
    // spacing (12) = 248px horizontally, 16px vertically. cursor_pos is
    // surface-relative, so subtract that inset to land the menu at the cursor.
    let x_inset = 248.0_f32;
    let y_inset = 16.0_f32;
    let menu_w = 200.0_f32; // matches the menu's fixed width
    let menu_h = 180.0_f32; // generous; clamp keeps it on-screen
    let avail_w = crate::position::layout::RIGHT_PANEL_WIDTH;
    let avail_h = crate::position::layout::WINDOW_HEIGHT - y_inset - 16.0;
    let cx = (cursor_pos.x - x_inset).max(0.0);
    let cy = (cursor_pos.y - y_inset).max(0.0);
    let px = cx.min((avail_w - menu_w - 8.0).max(8.0)).max(8.0);
    let py = cy.min((avail_h - menu_h - 8.0).max(8.0)).max(8.0);
    mouse_area(
        container(
            container(menu)
                .padding(cosmic::iced::Padding { top: py, left: px, right: 0.0, bottom: 0.0 })
                .width(Length::Fill)
                .height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
    )
    .on_press(SearchMessage::VaultCloseContextMenu)
    .into()
}

/// A single hidden-app tile: emoji placeholder + name, click to launch.
fn hidden_app_tile<'a>(app: &'a super::hidden_apps::HiddenApp) -> Element<'a, SearchMessage> {
    let label = if app.meta.name.chars().count() > 10 {
        format!("{}...", app.meta.name.chars().take(10).collect::<String>())
    } else {
        app.meta.name.clone()
    };
    let content = column![
        text("\u{1F512}").size(28), // lock glyph placeholder; real icons later
        text(label).size(12),
    ]
    .spacing(4)
    .align_x(Horizontal::Center);
    mouse_area(container(content).padding(6))
        .on_press(SearchMessage::LaunchHiddenApp(app.id.clone()))
        .on_right_press(SearchMessage::ShowHiddenMenu(app.id.clone()))
        .into()
}

/// Small menu shown under a hidden app when right-clicked: Launch / Remove.
fn hidden_app_menu<'a>(app: &'a super::hidden_apps::HiddenApp, cursor_pos: cosmic::iced::Point) -> Element<'a, SearchMessage> {
    let item = |label: &'static str, msg: SearchMessage| -> Element<'a, SearchMessage> {
        mouse_area(
            container(text(label).size(13))
                .padding([6, 10])
                .width(Length::Fill),
        )
        .on_press(msg)
        .into()
    };
    let menu_box = container(
        column![
            item("↗ Launch", SearchMessage::LaunchHiddenApp(app.id.clone())),
            item("📤 Remove from vault", SearchMessage::RemoveFromVault(app.id.clone())),
        ]
        .spacing(2),
    )
    .padding(6)
    .width(Length::Fixed(180.0))
    .style(|_: &Theme| container::Style {
        background: Some(Color::from_rgb8(30, 30, 38).into()),
        border: cosmic::iced::Border {
            radius: 0.0.into(),
            width: 1.0,
            color: Color::from_rgb8(90, 90, 110),
        },
        ..Default::default()
    });
    // Same right-panel inset as the file menu: cursor_pos is surface-relative,
    // subtract 248px horizontal / 16px vertical to land at the cursor.
    let x_inset = 248.0_f32;
    let y_inset = 16.0_f32;
    let menu_w = 180.0_f32;
    let menu_h = 90.0_f32;
    let avail_w = crate::position::layout::RIGHT_PANEL_WIDTH;
    let avail_h = crate::position::layout::WINDOW_HEIGHT - y_inset - 16.0;
    let cx = (cursor_pos.x - x_inset).max(0.0);
    let cy = (cursor_pos.y - y_inset).max(0.0);
    let px = cx.min((avail_w - menu_w - 8.0).max(8.0)).max(8.0);
    let py = cy.min((avail_h - menu_h - 8.0).max(8.0)).max(8.0);
    container(
        container(menu_box)
            .padding(cosmic::iced::Padding { top: py, left: px, right: 0.0, bottom: 0.0 })
            .width(Length::Fill)
            .height(Length::Fill)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Build the hidden-apps grid (4 columns), or None if there are none.
fn hidden_apps_grid<'a>(vault: &'a Vault) -> Option<Element<'a, SearchMessage>> {
    if vault.hidden_apps.is_empty() {
        return None;
    }
    let grid = vault.hidden_apps
        .chunks(4)
        .fold(column!().spacing(8), |col, chunk| {
            let mut grid_row = row!().spacing(8).width(Length::Fill);
            for app in chunk {
                grid_row = grid_row.push(hidden_app_tile(app));
            }
            col.push(grid_row)
        });
    let col = column![
        text("Hidden apps").size(13),
        space::vertical().height(Length::Fixed(6.0)),
        grid,
    ]
    .spacing(0)
    .width(Length::Fill);
    Some(col.into())
}