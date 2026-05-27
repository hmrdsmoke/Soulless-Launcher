// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

use std::convert::Infallible;

use crate::drawers::state::Drawer;
use crate::search::AppPicker;
use crate::search::ContextMenu;
use crate::search::DrawerEditModal;
use crate::search::Message as SearchMessage;
use crate::search::OpenDrawer;

use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::widget::{
    column, container, image, mouse_area, row, scrollable, space, text,
    text_input,
};
use cosmic::iced::{Color, Element, Length, Theme};

// DnD bridge
use cosmic::iced::widget::Themer;
use cosmic::iced::clipboard::mime::{AllowedMimeTypes, AsMimeTypes};
use cosmic::widget::dnd_destination;
use cosmic::widget::dnd_destination::dnd_destination_for_data;

use crate::position::layout::{TOOLBOX_WIDTH, RIGHT_PANEL_WIDTH};
const GRID_COLUMNS: usize = 4;
const ICON_SIZE: f32 = 64.0;

// Cap picker render to prevent the freeze — was building 200+ widget trees
// per frame. 50 is plenty; typing filters it down fast.
const PICKER_MAX_RENDER: usize = 50;

// ─────────────────────────────────────────────────────────────
// DnD payload type — carries an app ID as plain text
// ─────────────────────────────────────────────────────────────

const APP_MIME_TYPES: &[&str] = &[
    "text/plain;charset=utf-8",
    "text/plain",
    "UTF8_STRING",
    "STRING",
];

#[derive(Debug, Clone, Default)]
struct AppIdPayload(String);

impl AllowedMimeTypes for AppIdPayload {
    fn allowed() -> std::borrow::Cow<'static, [String]> {
        std::borrow::Cow::Owned(
            APP_MIME_TYPES.iter().map(|s| s.to_string()).collect(),
        )
    }
}

impl TryFrom<(Vec<u8>, String)> for AppIdPayload {
    type Error = Infallible;
    fn try_from(value: (Vec<u8>, String)) -> Result<Self, Self::Error> {
        Ok(AppIdPayload(
            String::from_utf8_lossy(&value.0).trim().to_string(),
        ))
    }
}

impl AsMimeTypes for AppIdPayload {
    fn available(&self) -> std::borrow::Cow<'static, [String]> {
        std::borrow::Cow::Owned(
            APP_MIME_TYPES.iter().map(|s| s.to_string()).collect(),
        )
    }
    fn as_bytes(&self, _mime: &str) -> Option<std::borrow::Cow<'static, [u8]>> {
        Some(std::borrow::Cow::Owned(self.0.clone().into_bytes()))
    }
}

// ─────────────────────────────────────────────────────────────
// Main View
// ─────────────────────────────────────────────────────────────

pub fn view<'a>(
    search: &'a crate::search::Search,
) -> Element<'a, SearchMessage> {
    let search_bar = text_input("Search all apps...", &search.query)
        .id(cosmic::widget::Id::new("soulless-search-bar"))
        .on_input(SearchMessage::QueryChanged)
        .on_submit(SearchMessage::SearchBarClicked)
        .padding(16)
        .size(18);

    let drawers_column = column(
        search
            .drawer_state
            .drawers()
            .iter()
            .map(|drawer| sidebar_drawer_button(search, drawer))
            .collect::<Vec<_>>(),
    )
    .spacing(6);

    let main_toolbox = column![
        container(search_bar).padding(16),
        drawers_column,
        container(
            mouse_area(
                container(
                    row![
                        text("➕").size(18),
                        space::horizontal().width(Length::Fixed(12.0)),
                        text("New Drawer").size(15),
                    ]
                    .align_y(Vertical::Center)
                    .padding(14)
                )
                .width(Length::Fill)
            )
            .on_press(SearchMessage::CreateDrawer)
        )
        .padding([8, 16]),
        container(
            mouse_area(
                container(
                    row![
                        text("🔒").size(20),
                        space::horizontal().width(Length::Fixed(12.0)),
                        text("Vault").size(16),
                    ]
                    .align_y(Vertical::Center)
                    .padding(14)
                )
                .width(Length::Fill)
            )
            .on_press(SearchMessage::VaultClicked)
        )
        .padding(16)
    ]
    .spacing(8)
    .width(Length::Fixed(TOOLBOX_WIDTH))
    .height(Length::Fill);

    let right_panel_content: Element<'a, SearchMessage> =
        if let Some(picker) = &search.app_picker {
            app_picker_view(search, picker)
        } else if search.show_search_results {
            search_results_view(search)
        } else {
            match &search.current_open_drawer {
                OpenDrawer::Pinned(name) => drawer_contents_view(search, name),
                OpenDrawer::Vault => crate::vault::ui::view(&search.vault),
                OpenDrawer::Search => container(
                    text("Search or select a drawer").size(18)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into(),
            }
        };

    let base = row![
        main_toolbox,
        space::horizontal().width(Length::Fixed(12.0)),
        container(right_panel_content)
            .width(Length::Fixed(RIGHT_PANEL_WIDTH))
            .height(Length::Fill)
            .padding(16),
    ]
    .spacing(0)
    .width(Length::Shrink)
    .height(Length::Fill);

    if let Some(modal) = &search.drawer_edit {
        let (title, placeholder, value) = match modal {
            DrawerEditModal::Rename { input, .. } => (
                "Rename Drawer",
                "New name…",
                input.as_str(),
            ),
            DrawerEditModal::SetIcon { input, .. } => (
                "Set Icon",
                "Paste an emoji…",
                input.as_str(),
            ),
        };

        let modal_widget = container(
            column![
                text(title).size(18),
                space::vertical().height(Length::Fixed(12.0)),
                text_input(placeholder, value)
                    .on_input(SearchMessage::DrawerEditInputChanged)
                    .on_submit(SearchMessage::DrawerEditConfirm)
                    .padding(12)
                    .size(16),
                space::vertical().height(Length::Fixed(16.0)),
                row![
                    mouse_area(
                        container(text("Save").size(14))
                            .padding([8, 20])
                            .style(|_: &Theme| container::Style {
                                background: Some(
                                    Color::from_rgb8(60, 120, 60).into()
                                ),
                                border: cosmic::iced::border::rounded(6),
                                ..Default::default()
                            })
                    )
                    .on_press(SearchMessage::DrawerEditConfirm),
                    space::horizontal().width(Length::Fixed(12.0)),
                    mouse_area(
                        container(text("Cancel").size(14))
                            .padding([8, 20])
                            .style(|_: &Theme| container::Style {
                                background: Some(
                                    Color::from_rgb8(80, 80, 80).into()
                                ),
                                border: cosmic::iced::border::rounded(6),
                                ..Default::default()
                            })
                    )
                    .on_press(SearchMessage::DrawerEditCancel),
                ]
                .align_y(Vertical::Center),
            ]
            .spacing(4)
        )
        .width(Length::Fixed(320.0))
        .padding(24)
        .style(|_: &Theme| container::Style {
            background: Some(Color::from_rgb8(40, 40, 50).into()),
            border: cosmic::iced::border::rounded(10),
            ..Default::default()
        });

        // Darken backdrop + center the modal
        let backdrop = mouse_area(
            container(
                container(base)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|_: &Theme| container::Style {
                        background: Some(
                            Color::from_rgba8(0, 0, 0, 0.5).into()
                        ),
                        ..Default::default()
                    })
            )
            .width(Length::Fill)
            .height(Length::Fill)
        )
        .on_press(SearchMessage::DrawerEditCancel);

        column![
            backdrop,
            container(modal_widget)
                .width(Length::Fill)
                .height(Length::Shrink)
                .center_x(Length::Fill)
                .padding([80, 0, 0, 0]),
        ]
        .into()
    } else if let Some(menu) = &search.context_menu {
        let menu_widget = context_menu_view(menu);
        let dismiss = mouse_area(
            container(base).width(Length::Fill).height(Length::Fill)
        )
        .on_press(SearchMessage::CloseContextMenu);
        column![dismiss, container(menu_widget).padding(8)].into()
    } else {
        base.into()
    }
}

// ─────────────────────────────────────────────────────────────
// Sidebar Drawer Button — DnD drop target for app icons
// ─────────────────────────────────────────────────────────────

fn sidebar_drawer_button<'a>(
    search: &'a crate::search::Search,
    drawer: &'a Drawer,
) -> Element<'a, SearchMessage> {
    let drawer_name = drawer.name.clone();

    let is_active = search.current_open_drawer
        == OpenDrawer::Pinned(drawer_name.clone());

    let is_drag_target = search.drag_hover_drawer.as_deref()
        == Some(drawer_name.as_str());

    // Show total item count (apps + files)
    let item_count = search.drawer_state.item_count(&drawer_name);

    let bg_color: Option<cosmic::iced::Background> = if is_drag_target {
        Some(Color::from_rgb8(40, 80, 40).into())
    } else if is_active {
        Some(Color::from_rgb8(60, 60, 80).into())
    } else {
        None
    };

    let border_color = if is_drag_target {
        Color::from_rgb8(60, 180, 60)
    } else {
        Color::from_rgba8(0, 0, 0, 0.0)
    };

    let dn_click = drawer_name.clone();
    let dn_rclick = drawer_name.clone();
    let dn_enter = drawer_name.clone();
    let dn_finish = drawer_name.clone();
    let icon_str = drawer.icon.clone();
    let name_str = drawer.name.clone();

    // Build the full button as a focusable cosmic::widget::button so
    // keyboard navigation (Tab / arrow keys) works on drawer buttons.
    let row_content = cosmic::iced::widget::row![
        cosmic::iced::widget::text(icon_str).size(18),
        cosmic::iced::widget::space::horizontal()
            .width(Length::Fixed(12.0)),
        cosmic::iced::widget::text(name_str).size(16),
        cosmic::iced::widget::space::horizontal()
            .width(Length::Fill),
        cosmic::iced::widget::text(item_count.to_string()).size(12),
    ]
    .align_y(Vertical::Center)
    .padding(14);

    let button: cosmic::Element<'_, SearchMessage> =
        cosmic::widget::button::custom(row_content)
            .width(Length::Fill)
            .on_press(SearchMessage::DrawerClicked(dn_click.clone()))
            .selected(is_active)
            .class(cosmic::theme::Button::MenuItem)
            .id(cosmic::widget::Id::new(format!("drawer-btn-{}", drawer.name)))
            .into();

    // Wrap the button as a dnd_destination_for_data.
    // When an AppIdPayload is dropped here we fire AppDroppedOnDrawer.
    let dest: cosmic::Element<'_, SearchMessage> =
        dnd_destination_for_data::<AppIdPayload, SearchMessage>(
            button,
            move |payload, _action| {
                let app_id = payload
                    .map(|p| p.0)
                    .unwrap_or_default();
                SearchMessage::AppDroppedOnDrawer(dn_finish.clone(), app_id)
            },
        )
        .on_enter(move |_x, _y, _mimes| {
            SearchMessage::DrawerDragHover(Some(dn_enter.clone()))
        })
        .on_leave(|| SearchMessage::DrawerDragHover(None))
        .into();

    // Bridge cosmic::Theme → cosmic::iced::Theme
    Themer::new(None::<cosmic::Theme>, dest).into()
}

// ─────────────────────────────────────────────────────────────
// Drawer Contents
// ─────────────────────────────────────────────────────────────

fn drawer_contents_view<'a>(
    search: &'a crate::search::Search,
    drawer_name: &'a str,
) -> Element<'a, SearchMessage> {
    let pinned_ids = search.drawer_state.apps_in_drawer(drawer_name);
    let drawer_files = search.drawer_state.files_in_drawer(drawer_name);

    let is_file_hover = search.drawer_file_hover.as_deref()
        == Some(drawer_name);

    let header = mouse_area(
        container(
            row![
                text(format!("📁  {drawer_name}")).size(22),
                space::horizontal().width(Length::Fill),
                text("Drag apps or files here · right-click to add").size(11),
            ]
            .padding([0, 0, 12, 0])
            .align_y(Vertical::Center)
        )
        .width(Length::Fill)
    )
    .on_right_press(
        SearchMessage::RightClickDrawerBackground(drawer_name.to_string())
    );

    let is_empty = pinned_ids.is_empty() && drawer_files.is_empty();

    // ── Build the inner content (empty state or grid) ─────────────────────
    let content: Element<'a, SearchMessage> = if is_empty {
        mouse_area(
            container(
                column![
                    header,
                    space::vertical().height(Length::Fixed(32.0)),
                    container(
                        column![
                            text("📂").size(48),
                            space::vertical().height(Length::Fixed(8.0)),
                            text("This drawer is empty.").size(16),
                            text("Drop files here, or right-click to add apps.").size(13),
                        ]
                        .spacing(8)
                        .align_x(Horizontal::Center)
                    )
                    .width(Length::Fill)
                    .center_x(Length::Fill),
                ]
            )
            .width(Length::Fill)
            .height(Length::Fill)
        )
        .on_right_press(
            SearchMessage::RightClickDrawerBackground(drawer_name.to_string())
        )
        .into()
    } else {
        let app_entries: Vec<_> = pinned_ids
            .iter()
            .filter_map(|id| search.app_by_id(id).map(|app| (id, app)))
            .collect();

        let mut content_col = column!().spacing(8);

        // ── Apps grid ─────────────────────────────────────────────────────
        if !app_entries.is_empty() {
            let apps_grid = app_entries
                .chunks(GRID_COLUMNS)
                .fold(column!().spacing(8), |col, chunk| {
                    let mut grid_row = row!().spacing(8).width(Length::Fill);
                    for (app_id, app) in chunk {
                        grid_row = grid_row.push(
                            drawer_app_icon(app, drawer_name, app_id)
                        );
                    }
                    col.push(grid_row)
                });
            content_col = content_col.push(apps_grid);
        }

        // ── Files grid ────────────────────────────────────────────────────
        if !drawer_files.is_empty() {
            if !app_entries.is_empty() {
                content_col = content_col.push(
                    container(text("Files").size(11)).padding([8, 0, 4, 0])
                );
            }

            let files_grid = drawer_files
                .chunks(GRID_COLUMNS)
                .fold(column!().spacing(8), |col, chunk| {
                    let mut grid_row = row!().spacing(8).width(Length::Fill);
                    for file in chunk {
                        grid_row = grid_row.push(
                            drawer_file_icon(file, drawer_name)
                        );
                    }
                    col.push(grid_row)
                });

            content_col = content_col.push(files_grid);
        }

        mouse_area(
            container(column![header, scrollable(content_col)])
                .width(Length::Fill)
                .height(Length::Fill)
        )
        .on_right_press(
            SearchMessage::RightClickDrawerBackground(drawer_name.to_string())
        )
        .into()
    };

    // ── Drop zone — mirrors vault_ui.rs exactly ───────────────────────────
    //
    // Step 1: inner widget as cosmic::Element (cosmic::widget::container)
    let drop_inner: cosmic::Element<'_, SearchMessage> =
        cosmic::widget::container(
            cosmic::widget::text(
                if is_file_hover { "Drop to add files" } else { "Drop files here to add them" }
            ).size(11),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

    // Step 2: dnd_destination wraps drop_inner → cosmic::Element.
    // Registers the widget bounds with the compositor as a drop target.
    let dn_enter  = drawer_name.to_string();
    let dn_finish = drawer_name.to_string();

    let drop_dest: cosmic::Element<'_, SearchMessage> =
        cosmic::widget::dnd_destination(
            drop_inner,
            vec![std::borrow::Cow::Borrowed("text/uri-list")],
        )
        .on_enter(move |_x, _y, _mimes| {
            SearchMessage::DrawerFileHover(Some(dn_enter.clone()))
        })
        .on_leave(|| SearchMessage::DrawerFileHover(None))
        .on_finish(move |_mime, data, _action, _x, _y| {
            let payload = String::from_utf8_lossy(&data);
            let paths = payload
                .lines()
                .map(str::trim)
                .filter(|l| l.starts_with("file://"))
                .filter_map(|l| {
                    let raw = l.trim_start_matches("file://");
                    let decoded = uri_decode(raw);
                    let p = std::path::PathBuf::from(decoded);
                    if p.exists() { Some(p) } else { None }
                })
                .collect::<Vec<_>>();
            SearchMessage::FilesDroppedOnDrawer(dn_finish.clone(), paths)
        })
        .into();

    // Step 3: Themer bridges cosmic::Theme → cosmic::iced::Theme.
    // Outer iced container applies the hover highlight styling on top.
    let (drop_bg, drop_border_color, drop_border_width) = if is_file_hover {
        (
            Color::from_rgba8(30, 80, 30, 0.35),
            Color::from_rgb8(60, 200, 60),
            2.0_f32,
        )
    } else {
        (
            Color::from_rgba8(255, 255, 255, 0.03),
            Color::from_rgba8(255, 255, 255, 0.08),
            1.0_f32,
        )
    };

    // ── Final layout: content above, drop zone strip below ────────────────
    mouse_area(
        container(
            column![
                content,
                container(Themer::new(None::<cosmic::Theme>, drop_dest))
                    .width(Length::Fill)
                    .style(move |_: &Theme| container::Style {
                        background: Some(drop_bg.into()),
                        border: cosmic::iced::Border {
                            color: drop_border_color,
                            width: drop_border_width,
                            radius: cosmic::iced::border::rounded(10).radius,
                        },
                        ..Default::default()
                    }),
            ]
            .spacing(8)
        )
        .width(Length::Fill)
        .height(Length::Fill)
    )
    .on_right_press(
        SearchMessage::RightClickDrawerBackground(drawer_name.to_string())
    )
    .into()
}

// ─────────────────────────────────────────────────────────────
// Drawer App Icon
// ─────────────────────────────────────────────────────────────

fn drawer_app_icon<'a>(
    app: &'a crate::search::indexer::AppEntry,
    drawer_name: &'a str,
    app_id: &'a str,
) -> Element<'a, SearchMessage> {
    let icon_widget = image(&app.icon_path)
        .width(Length::Fixed(ICON_SIZE))
        .height(Length::Fixed(ICON_SIZE));

    let label = text(truncate_label(&app.name, 12)).size(12);

    let content = column![icon_widget, label]
        .spacing(4)
        .align_x(Horizontal::Center);

    mouse_area(container(content).padding(6))
        .on_press(SearchMessage::AppClicked(app.exec.clone()))
        .on_right_press(SearchMessage::RightClickDrawerApp(
            drawer_name.to_string(),
            app_id.to_string(),
        ))
        .into()
}

// ─────────────────────────────────────────────────────────────
// Drawer File Icon
// ─────────────────────────────────────────────────────────────

fn drawer_file_icon<'a>(
    file: &'a crate::drawers::state::DrawerFile,
    drawer_name: &'a str,
) -> Element<'a, SearchMessage> {
    let emoji = file_emoji(&file.name);

    let icon_cell = container(
        container(text(emoji).size(18))
            .width(Length::Fixed(ICON_SIZE))
            .height(Length::Fixed(ICON_SIZE))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_| container::Style {
                background: Some(
                    cosmic::iced::Color::from_rgba8(60, 60, 80, 0.9).into(),
                ),
                border: cosmic::iced::Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: cosmic::iced::Color::from_rgb8(90, 90, 110),
                },
                ..Default::default()
            }),
    );

    let label = text(truncate_label(&file.name, 12)).size(12);

    let content = column![icon_cell, label]
        .spacing(4)
        .align_x(Horizontal::Center);

    mouse_area(container(content).padding(6))
        .on_press(SearchMessage::OpenDrawerFile(file.path.clone()))
        .on_right_press(SearchMessage::RightClickDrawerFile(
            drawer_name.to_string(),
            file.path.clone(),
        ))
        .into()
}

/// Pick a representative emoji for a file by extension.
fn file_emoji(name: &str) -> &'static str {
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "pdf"                               => "📄",
        "doc" | "docx" | "odt" | "rtf"     => "📝",
        "xls" | "xlsx" | "ods" | "csv"     => "📊",
        "ppt" | "pptx" | "odp"             => "📋",
        "txt" | "md" | "rst"               => "📃",
        "png" | "jpg" | "jpeg" | "gif"
            | "webp" | "bmp" | "svg"       => "🖼",
        "mp4" | "mkv" | "avi" | "mov"
            | "webm"                        => "🎬",
        "mp3" | "flac" | "ogg" | "wav"
            | "aac" | "m4a"                => "🎵",
        "zip" | "tar" | "gz" | "xz"
            | "7z" | "rar"                 => "🗜",
        "rs" | "py" | "js" | "ts"
            | "go" | "c" | "cpp" | "h"
            | "java" | "kt" | "swift"      => "💻",
        "sh" | "bash" | "zsh" | "fish"     => "⚙",
        "json" | "toml" | "yaml" | "yml"
            | "xml" | "ini" | "conf"       => "🔧",
        _                                   => "📁",
    }
}

// ─────────────────────────────────────────────────────────────
// Context Menus
// ─────────────────────────────────────────────────────────────

fn context_menu_view<'a>(menu: &'a ContextMenu) -> Element<'a, SearchMessage> {
    match menu {
        ContextMenu::DrawerBackground { drawer } => container(
            column![
                menu_item("➕ Add Apps", SearchMessage::OpenAppPicker(drawer.clone())),
                menu_divider(),
                menu_item("🗑 Clear Drawer", SearchMessage::ClearDrawer(drawer.clone())),
            ]
            .spacing(2)
        )
        .style(context_menu_style)
        .padding(8)
        .width(Length::Fixed(260.0))
        .into(),

        ContextMenu::DrawerSidebar { drawer } => container(
            column![
                menu_item("✏ Rename", SearchMessage::OpenRenameDrawer(drawer.clone())),
                menu_item("🎨 Set Icon", SearchMessage::OpenSetIconDrawer(drawer.clone())),
                menu_divider(),
                menu_item("⬆ Move Up", SearchMessage::MoveDrawerUp(drawer.clone())),
                menu_item("⬇ Move Down", SearchMessage::MoveDrawerDown(drawer.clone())),
                menu_divider(),
                menu_item("🗑 Delete Drawer", SearchMessage::DeleteDrawer(drawer.clone())),
            ]
            .spacing(2)
        )
        .style(context_menu_style)
        .padding(8)
        .width(Length::Fixed(260.0))
        .into(),

        ContextMenu::DrawerApp { drawer, app_id } => container(
            column![
                menu_item(
                    "✖ Remove App",
                    SearchMessage::RemoveAppFromDrawer(drawer.clone(), app_id.clone()),
                ),
            ]
            .spacing(2)
        )
        .style(context_menu_style)
        .padding(8)
        .width(Length::Fixed(260.0))
        .into(),

        ContextMenu::DrawerFile { drawer, file_path } => container(
            column![
                menu_item(
                    "↗ Open File",
                    SearchMessage::OpenDrawerFile(file_path.clone()),
                ),
                menu_divider(),
                menu_item(
                    "✖ Remove from Drawer",
                    SearchMessage::RemoveFileFromDrawer(drawer.clone(), file_path.clone()),
                ),
            ]
            .spacing(2)
        )
        .style(context_menu_style)
        .padding(8)
        .width(Length::Fixed(260.0))
        .into(),
    }
}

fn menu_item<'a>(label: &'a str, msg: SearchMessage) -> Element<'a, SearchMessage> {
    mouse_area(
        container(text(label).size(14))
            .padding([8, 12])
            .width(Length::Fill)
    )
    .on_press(msg)
    .into()
}

fn menu_divider<'a>() -> Element<'a, SearchMessage> {
    container(space::vertical().height(Length::Fixed(1.0)))
        .width(Length::Fill)
        .style(|_: &Theme| container::Style {
            background: Some(Color::from_rgb8(70, 70, 70).into()),
            ..Default::default()
        })
        .into()
}

fn context_menu_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgb8(45, 45, 55).into()),
        border: cosmic::iced::border::rounded(8),
        ..Default::default()
    }
}

// ─────────────────────────────────────────────────────────────
// Search Results — app icons are drag sources
// ─────────────────────────────────────────────────────────────

fn search_results_view<'a>(
    search: &'a crate::search::Search,
) -> Element<'a, SearchMessage> {
    let apps: Vec<_> = search
        .filtered_apps()
        .iter()
        .filter_map(|i| search.app(*i))
        .collect();

    let grid = apps
        .chunks(GRID_COLUMNS)
        .fold(column!().spacing(8), |col, chunk| {
            let mut grid_row = row!().spacing(8);
            for app in chunk {
                grid_row = grid_row.push(app_icon_button(app));
            }
            col.push(grid_row)
        });

    scrollable(grid).into()
}

// ─────────────────────────────────────────────────────────────
// Search App Icon — drag source + click to launch
// ─────────────────────────────────────────────────────────────

fn app_icon_button<'a>(
    app: &'a crate::search::indexer::AppEntry,
) -> Element<'a, SearchMessage> {
    let exec = app.exec.clone();

    let content = column![
        image(&app.icon_path)
            .width(Length::Fixed(ICON_SIZE))
            .height(Length::Fixed(ICON_SIZE)),
        text(truncate_label(&app.name, 12)).size(12),
    ]
    .spacing(4)
    .align_x(Horizontal::Center);

    mouse_area(container(content).padding(6))
        .on_press(SearchMessage::AppClicked(exec))
        .into()
}

// ─────────────────────────────────────────────────────────────
// App Picker — capped at PICKER_MAX_RENDER to prevent freeze
// ─────────────────────────────────────────────────────────────

fn app_picker_view<'a>(
    search: &'a crate::search::Search,
    picker: &'a AppPicker,
) -> Element<'a, SearchMessage> {
    let search_input = text_input("Search apps...", &picker.query)
        .on_input(SearchMessage::AppPickerQueryChanged)
        .padding(12);

    // Only render first PICKER_MAX_RENDER items — full list was 200+ which
    // caused the freeze by building thousands of widgets per frame.
    let total = picker.filtered.len();
    let apps: Vec<_> = picker
        .filtered
        .iter()
        .take(PICKER_MAX_RENDER)
        .filter_map(|i| search.app(*i))
        .collect();

    let list = apps.into_iter().fold(column!().spacing(6), |col, app| {
        col.push(
            mouse_area(
                container(
                    row![
                        text(&app.name),
                        space::horizontal().width(Length::Fill),
                        text("➕"),
                    ]
                    .align_y(Vertical::Center)
                    .padding(12)
                )
            )
            .on_press(SearchMessage::AddAppToDrawer(
                picker.drawer.clone(),
                app.id.clone(),
            ))
        )
    });

    let mut content_col = column![
        row![
            text("Add Apps").size(22),
            space::horizontal().width(Length::Fill),
            mouse_area(text("✖")).on_press(SearchMessage::CloseAppPicker),
        ],
        search_input,
        scrollable(list),
    ]
    .spacing(12);

    if total > PICKER_MAX_RENDER {
        content_col = content_col.push(
            text(format!(
                "Showing {} of {} — type to filter",
                PICKER_MAX_RENDER, total
            ))
            .size(11),
        );
    }

    container(content_col)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// ─────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────

fn truncate_label(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_string()
    } else {
        format!("{}…", text.chars().take(max).collect::<String>())
    }
}

// ── Drawer file-drop highlight styles ────────────────────────────────────────
// Named fn so we can pass it as a fn pointer to cosmic::widget::container::style().
// ── URI percent-decoding (used by drawer file drop zone) ─────────────────────

fn uri_decode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (
                hex_nibble(bytes[i + 1]),
                hex_nibble(bytes[i + 2]),
            ) {
                out.push((hi << 4 | lo) as char);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

// === DONE ===
// drawer_contents_view: renders apps grid + files grid in same view :: done
// Files section label shown only when both apps and files present :: done
// drawer_file_icon(): emoji by extension, click = OpenDrawerFile, right-click = context menu :: done
// file_emoji(): extension-to-emoji mapping covering common file types :: done
// ContextMenu::DrawerFile: Open File + Remove from Drawer :: done
// sidebar badge: item_count() = apps + files instead of apps only :: done
// All existing app drag/drop, context menus, picker, modals unchanged :: done
// Added file drop zone to drawer_contents_view — mirrors vault_ui.rs exactly :: done
// Step 1: drop_inner as cosmic::Element via cosmic::widget::container :: done
// Step 2: cosmic::widget::dnd_destination wraps drop_inner :: done
//   on_enter → DrawerFileHover(Some(name)), on_leave → DrawerFileHover(None) :: done
//   on_finish → parse text/uri-list → FilesDroppedOnDrawer :: done
// Step 3: outer iced container(Themer::new(None, drop_dest)) applies hover style :: done
// uri_decode() helper for percent-encoded file:// URIs :: done
pub mod state;
