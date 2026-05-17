// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

use crate::search::Message as SearchMessage;
use crate::search::OpenDrawer;

use cosmic::iced::widget::{
    column,
    container,
    image,
    mouse_area,
    row,
    scrollable,
    space,
    text,
    text_input,
};

use cosmic::iced::{Element, Length};

const TOOLBOX_WIDTH: f32 = 360.0;
const RIGHT_PANEL_WIDTH: f32 = 560.0;
const GRID_COLUMNS: usize = 4;
const ICON_SIZE: f32 = 64.0;

const FALLBACK_ICON: &str = "assets/launcher.png";

pub fn view<'a>(
    search: &'a crate::search::Search,
) -> Element<'a, SearchMessage> {
    let search_bar = text_input(
        "Search all apps...",
        &search.query,
    )
    .on_input(SearchMessage::QueryChanged)
    .padding(16)
    .size(18);

    let main_toolbox = column![
        container(search_bar).padding(16),

        column(
            search.drawers().iter().map(|name| {
                mouse_area(
                    container(
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
                        .padding(14)
                    )
                    .width(Length::Fill)
                )
                .on_press(
                    SearchMessage::DrawerClicked(name.clone())
                )
                .into()
            })
        )
        .spacing(6),

        container(
            mouse_area(
                container(
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
        if search.show_search_results {
            let results = search.filtered_apps();

            if results.is_empty() {
                container(
                    text("No apps found").size(16)
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
                            let mut grid_row = row!()
                                .spacing(8)
                                .width(Length::Fill);

                            for app in chunk {
                                grid_row =
                                    grid_row.push(app_icon_button(app));
                            }

                            for _ in chunk.len()..GRID_COLUMNS {
                                grid_row = grid_row.push(
                                    container(
                                        space::horizontal()
                                            .width(
                                                Length::Fixed(0.0)
                                            )
                                    )
                                    .width(Length::Fill)
                                );
                            }

                            col.push(grid_row)
                        }
                    );

                container(
                    scrollable(
                        container(grid)
                            .padding([0, 12, 0, 0])
                    )
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
                            text(format!(
                                "Drawer: {name}"
                            ))
                            .size(24),

                            text(
                                "Pinned drawer content goes here."
                            )
                            .size(16),
                        ]
                        .spacing(12)
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
                            text("Vault").size(24),

                            text("Vault UI goes here.")
                                .size(16),
                        ]
                        .spacing(12)
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
                            "Search or select a drawer"
                        )
                        .size(18)
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
        .padding(16);

    row![
        main_toolbox,

        space::horizontal()
            .width(Length::Fixed(12.0)),

        right_panel,
    ]
    .spacing(0)
    .width(Length::Shrink)
    .height(Length::Fill)
    .into()
}

fn app_icon_button<'a>(
    app: &'a crate::search::AppEntry,
) -> Element<'a, SearchMessage> {
    let icon_path = app
        .icon
        .as_deref()
        .and_then(resolve_icon_path)
        .unwrap_or_else(|| FALLBACK_ICON.to_string());

    let icon_widget = image(icon_path)
        .width(Length::Fixed(ICON_SIZE))
        .height(Length::Fixed(ICON_SIZE));

    let label = text(
        truncate_label(&app.name, 12)
    )
    .size(12)
    .center();

    let content = column![
        icon_widget,
        label,
    ]
    .spacing(4)
    .align_x(
        cosmic::iced::alignment::Horizontal::Center
    )
    .width(Length::Fill);

    Element::from(
        mouse_area(
            container(content)
                .padding(6)
                .width(Length::Fill)
        )
        .on_press(
            SearchMessage::AppClicked(
                app.exec.clone()
            )
        )
    )
}

fn resolve_icon_path(
    icon_name: &str,
) -> Option<String> {
    if icon_name.starts_with('/') {
        if std::path::Path::new(icon_name).exists() {
            return Some(icon_name.to_string());
        }

        return None;
    }

    let search_dirs = [
        "/usr/share/icons/hicolor/256x256/apps",
        "/usr/share/icons/hicolor/128x128/apps",
        "/usr/share/icons/hicolor/64x64/apps",
        "/usr/share/icons/hicolor/48x48/apps",
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
// Replaced heavy button widgets with lightweight mouse_area :: done
// Direct clickable icons :: done
// Added fallback icon support :: done
// Added icon path resolution :: done
// Removed expensive button styling :: done
// Grid layout preserved :: done
// Faster rendering + snappier scrolling :: done