// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

use crate::search::Message as SearchMessage;
use crate::search::OpenDrawer;

use cosmic::iced::widget::{
    button,
    column,
    container,
    image,
    row,
    scrollable,
    space,
    text,
    text_input,
};

use cosmic::iced::{Color, Element, Length};

const TOOLBOX_WIDTH: f32 = 360.0;
const RIGHT_PANEL_WIDTH: f32 = 560.0;
const GRID_COLUMNS: usize = 4;
const ICON_SIZE: f32 = 64.0;

const FALLBACK_ICON: &str = "assets/launcher.png";

pub fn view<'a>(
    search: &'a crate::search::Search,
) -> Element<'a, SearchMessage> {
    // =========================================================
    // Search Bar
    // =========================================================

    // IMPORTANT:
    // COSMIC iced DOES NOT support:
    // - text_input::Id
    // - text_input::focus
    // - .on_press() on TextInput
    //
    // So we use the compatible version only.

    let search_bar = text_input(
        "Search all apps...",
        &search.query,
    )
    .on_input(SearchMessage::QueryChanged)
    .padding(16)
    .size(18);

    // =========================================================
    // Left Toolbox
    // =========================================================

    let drawer_column = column(
        search.drawers().iter().map(|name| {
            button(
                row![
                    text("📁").size(18),

                    space::horizontal()
                        .width(Length::Fixed(12.0)),

                    text(name.clone()).size(16),

                    space::horizontal()
                        .width(Length::Fill),

                    text("→").size(14),
                ]
                .align_y(
                    cosmic::iced::alignment::Vertical::Center
                )
                .padding(14),
            )
            .width(Length::Fill)
            .height(Length::Fixed(58.0))
            .style(|_theme, _status| button::Style {
                background: Some(
                    Color::from_rgb8(40, 40, 45).into(),
                ),
                border: cosmic::iced::border::rounded(8),
                ..Default::default()
            })
            .on_press(
                SearchMessage::DrawerClicked(
                    name.clone(),
                ),
            )
            .into()
        }),
    )
    .spacing(6);

    let vault_button = container(
        button(
            row![
                text("🔒").size(20),

                space::horizontal()
                    .width(Length::Fixed(12.0)),

                text("Vault (Secure Folder)")
                    .size(16),
            ]
            .align_y(
                cosmic::iced::alignment::Vertical::Center
            )
            .padding(14),
        )
        .width(Length::Fill)
        .height(Length::Fixed(68.0))
        .style(|_theme, _status| button::Style {
            background: Some(
                Color::from_rgb8(28, 28, 38).into(),
            ),
            border: cosmic::iced::border::rounded(8),
            ..Default::default()
        })
        .on_press(SearchMessage::VaultClicked),
    )
    .padding(16);

    let main_toolbox = column![
        container(search_bar)
            .padding(16),

        drawer_column,

        vault_button
    ]
    .spacing(8)
    .width(Length::Fixed(TOOLBOX_WIDTH))
    .height(Length::Fill);

    // =========================================================
    // Right Panel
    // =========================================================

    let right_panel_content: Element<'a, SearchMessage> =
        if search.show_search_results {
            let results = search.filtered_apps();

            if results.is_empty() {
                container(
                    text("No apps found").size(16),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
            } else {
                let app_entries: Vec<_> = results
                    .iter()
                    .filter_map(|&index| search.app(index))
                    .collect();

                let grid = app_entries
                    .chunks(GRID_COLUMNS)
                    .fold(
                        column!().spacing(8),
                        |col, chunk| {
                            let mut grid_row = row![]
                                .spacing(8)
                                .width(Length::Fill);

                            for app in chunk {
                                grid_row = grid_row.push(
                                    app_icon_button(app),
                                );
                            }

                            // Pad incomplete rows
                            for _ in chunk.len()..GRID_COLUMNS {
                                grid_row = grid_row.push(
                                    container(
                                        space::horizontal(),
                                    )
                                    .width(Length::Fill),
                                );
                            }

                            col.push(grid_row)
                        },
                    );

                container(
                    scrollable(
                        container(grid)
                            .padding([0, 12, 0, 0]),
                    ),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
        } else {
            match &search.current_open_drawer {
                OpenDrawer::Pinned(name) => {
                    container(
                        column![
                            text(
                                format!(
                                    "Drawer: {name}"
                                )
                            )
                            .size(24),

                            text(
                                "Pinned drawer content goes here."
                            )
                            .size(16),
                        ]
                        .spacing(12),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
                }

                OpenDrawer::Vault => {
                    container(
                        column![
                            text("Vault")
                                .size(24),

                            text(
                                "Vault UI goes here."
                            )
                            .size(16),
                        ]
                        .spacing(12),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
                }

                OpenDrawer::Search => {
                    container(
                        text(
                            "Search or select a drawer",
                        )
                        .size(18),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
                }
            }
        };

    let right_panel = container(right_panel_content)
        .width(Length::Fixed(RIGHT_PANEL_WIDTH))
        .height(Length::Fill)
        .padding(16)
        .style(|_| container::Style {
            background: Some(
                Color::from_rgb8(24, 24, 28).into(),
            ),
            border: cosmic::iced::border::rounded(8),
            ..Default::default()
        });

    row![
        main_toolbox,

        space::horizontal()
            .width(Length::Fixed(12.0)),

        right_panel
    ]
    .spacing(0)
    .width(Length::Shrink)
    .height(Length::Fill)
    .into()
}

// =========================================================
// App Icon Button
// =========================================================

fn app_icon_button<'a>(
    app: &'a crate::search::AppEntry,
) -> Element<'a, SearchMessage> {
    let icon_path = app
    .icon
    .as_ref()
    .and_then(|icon| resolve_icon_path(icon))
    .unwrap_or_else(|| FALLBACK_ICON.to_string());

    let icon_widget = container(
        image(icon_path)
            .width(Length::Fixed(ICON_SIZE))
            .height(Length::Fixed(ICON_SIZE)),
    )
    .width(Length::Fixed(ICON_SIZE))
    .height(Length::Fixed(ICON_SIZE));

    let label = text(
        truncate_label(&app.name, 12),
    )
    .size(12)
    .center();

    let cell = column![
        icon_widget,
        label
    ]
    .spacing(4)
    .align_x(
        cosmic::iced::alignment::Horizontal::Center
    )
    .width(Length::Fill);

    button(cell)
        .width(Length::Fill)
        .padding(8)
        .style(|_theme, _status| button::Style {
            background: Some(
                Color::from_rgb8(35, 35, 40).into(),
            ),
            border: cosmic::iced::border::rounded(10),
            ..Default::default()
        })
        .on_press(
            SearchMessage::AppClicked(
                app.exec.clone(),
            ),
        )
        .into()
}

// =========================================================
// Icon Resolution
// =========================================================

fn resolve_icon_path(
    icon_name: &str,
) -> Option<String> {
    // Already absolute
    if icon_name.starts_with('/') {
        if std::path::Path::new(icon_name).exists() {
            return Some(icon_name.to_string());
        }

        return None;
    }

    let search_dirs = [
        "/usr/share/icons/hicolor/64x64/apps",
        "/usr/share/icons/hicolor/48x48/apps",
        "/usr/share/icons/hicolor/128x128/apps",
        "/usr/share/icons/hicolor/scalable/apps",
        "/usr/share/pixmaps",
    ];

    let extensions = [
        "png",
        "svg",
        "xpm",
    ];

    for dir in &search_dirs {
        for ext in &extensions {
            let path = format!(
                "{}/{}.{}",
                dir,
                icon_name,
                ext
            );

            if std::path::Path::new(&path).exists() {
                return Some(path);
            }
        }
    }

    None
}

// =========================================================
// Label Truncation
// =========================================================

fn truncate_label(
    name: &str,
    max_chars: usize,
) -> String {
    let count = name.chars().count();

    if count <= max_chars {
        return name.to_string();
    }

    let truncated: String = name
        .chars()
        .take(max_chars.saturating_sub(1))
        .collect();

    format!("{truncated}…")
}

// === DONE ===
// Removed incompatible iced APIs :: done
// Removed text_input::Id :: done
// Removed text_input::focus :: done
// Removed TextInput .on_press() :: done
// Restored COSMIC iced compatibility :: done
// Preserved 4-wide app grid :: done
// Preserved icon resolution system :: done
// Preserved vault button :: done
// Preserved launcher styling :: done