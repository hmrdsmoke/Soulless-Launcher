// MIT License - see LICENSE file for full terms
//
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Grok (xAI).
// Do not remove these comments.

use crate::search::ContextMenu;
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

use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Color, Element, Length, Theme};

const TOOLBOX_WIDTH: f32 = 360.0;
const RIGHT_PANEL_WIDTH: f32 = 560.0;
const GRID_COLUMNS: usize = 4;
const ICON_SIZE: f32 = 64.0;

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
                let is_active = search.current_open_drawer
                    == OpenDrawer::Pinned(name.clone());

                let app_count = search
                    .drawer_state
                    .app_count(name);

                let count_label = if app_count > 0 {
                    format!("{app_count}")
                } else {
                    String::new()
                };

                mouse_area(
                    container(
                        row![
                            text("📁").size(18),

                            space::horizontal()
                                .width(Length::Fixed(12.0)),

                            text(name.clone()).size(16),

                            space::horizontal()
                                .width(Length::Fill),

                            text(count_label).size(12),

                            space::horizontal()
                                .width(Length::Fixed(8.0)),

                            text("→").size(14),
                        ]
                        .align_y(Vertical::Center)
                        .padding(14)
                    )
                    .width(Length::Fill)
                    .style(move |_: &Theme| container::Style {
                        background: if is_active {
                            Some(Color::from_rgb8(60, 60, 80).into())
                        } else {
                            None
                        },
                        border: cosmic::iced::border::rounded(6),
                        ..Default::default()
                    })
                )
                .on_press(
                    SearchMessage::DrawerClicked(name.clone())
                )
                .into()
            }).collect::<Vec<_>>()
        )
        .spacing(6),

        container(
            mouse_area(
                container(
                    row![
                        text("🔒").size(20),

                        space::horizontal()
                            .width(Length::Fixed(12.0)),

                        text("Vault (Secure Folder)").size(16),
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

    // ── Right panel ───────────────────────────────────────────────────────

    let right_panel_content: Element<'a, SearchMessage> =
        if let Some(picker) = &search.app_picker {
            app_picker_view(search, picker)
        } else if search.show_search_results {
            search_results_view(search)
        } else {
            match &search.current_open_drawer {
                OpenDrawer::Pinned(name) => {
                    drawer_contents_view(search, name)
                }

                OpenDrawer::Vault => {
                    container(
                        column![
                            text("Vault").size(24),
                            text("Vault UI goes here.").size(16),
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
                        text("Search or select a drawer").size(18)
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
                }
            }
        };

    // ── Context menu overlay ──────────────────────────────────────────────

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

    if let Some(menu) = &search.context_menu {
        let menu_widget = context_menu_view(menu);

        let dismiss = mouse_area(
            container(base)
                .width(Length::Fill)
                .height(Length::Fill)
        )
        .on_press(SearchMessage::CloseContextMenu);

        column![
            dismiss,
            container(menu_widget).padding(8),
        ]
        .into()
    } else {
        base.into()
    }
}

// ── Drawer contents view ──────────────────────────────────────────────────────

fn drawer_contents_view<'a>(
    search: &'a crate::search::Search,
    drawer_name: &'a str,
) -> Element<'a, SearchMessage> {
    let pinned_ids = search
        .drawer_state
        .apps_in_drawer(drawer_name);

    let header = mouse_area(
        container(
            row![
                text(format!("📁  {drawer_name}")).size(22),
                space::horizontal().width(Length::Fill),
                text("Right-click to add apps").size(12),
            ]
            .padding([0, 0, 12, 0])
            .align_y(Vertical::Center)
        )
        .width(Length::Fill)
    )
    .on_right_press(
        SearchMessage::RightClickDrawerBackground(
            drawer_name.to_string()
        )
    );

    if pinned_ids.is_empty() {
        return mouse_area(
            container(
                column![
                    header,
                    space::vertical().height(Length::Fixed(32.0)),
                    container(
                        column![
                            text("This drawer is empty.").size(16),
                            space::vertical()
                                .height(Length::Fixed(8.0)),
                            text("Right-click anywhere to add apps.")
                                .size(13),
                        ]
                        .align_x(Horizontal::Center)
                        .spacing(4)
                    )
                    .width(Length::Fill)
                    .center_x(Length::Fill),
                ]
                .spacing(0)
            )
            .width(Length::Fill)
            .height(Length::Fill)
        )
        .on_right_press(
            SearchMessage::RightClickDrawerBackground(
                drawer_name.to_string()
            )
        )
        .into();
    }

    let app_entries: Vec<_> = pinned_ids
        .iter()
        .filter_map(|id| {
            search.app_by_id(id).map(|app| (id, app))
        })
        .collect();

    let grid = app_entries
        .chunks(GRID_COLUMNS)
        .fold(column!().spacing(8), |col, chunk| {
            let mut grid_row =
                row!().spacing(8).width(Length::Fill);

            for (app_id, app) in chunk {
                grid_row = grid_row.push(
                    drawer_app_icon(app, drawer_name, app_id)
                );
            }

            for _ in chunk.len()..GRID_COLUMNS {
                grid_row = grid_row.push(
                    container(
                        space::horizontal().width(Length::Fixed(0.0))
                    )
                    .width(Length::Fill)
                );
            }

            col.push(grid_row)
        });

    mouse_area(
        container(
            column![
                header,
                scrollable(
                    container(grid).padding([0, 12, 0, 0])
                ),
            ]
            .spacing(8)
        )
        .width(Length::Fill)
        .height(Length::Fill)
    )
    .on_right_press(
        SearchMessage::RightClickDrawerBackground(
            drawer_name.to_string()
        )
    )
    .into()
}

// ── Individual drawer app icon ────────────────────────────────────────────────

fn drawer_app_icon<'a>(
    app: &'a crate::indexer::AppEntry,
    drawer_name: &'a str,
    app_id: &'a str,
) -> Element<'a, SearchMessage> {
    let icon_widget = image(&app.icon_path)
        .width(Length::Fixed(ICON_SIZE))
        .height(Length::Fixed(ICON_SIZE));

    let label = text(truncate_label(&app.name, 12))
        .size(12)
        .center();

    let content = column![icon_widget, label]
        .spacing(4)
        .align_x(Horizontal::Center)
        .width(Length::Fill);

    let drawer_for_right = drawer_name.to_string();
    let app_id_for_right = app_id.to_string();

    Element::from(
        mouse_area(
            container(content).padding(6).width(Length::Fill)
        )
        .on_press(SearchMessage::AppClicked(app.exec.clone()))
        .on_right_press(SearchMessage::RightClickDrawerApp(
            drawer_for_right,
            app_id_for_right,
        ))
    )
}

// ── Context menu ─────────────────────────────────────────────────────────────

fn context_menu_view<'a>(
    menu: &'a ContextMenu,
) -> Element<'a, SearchMessage> {
    match menu {
        ContextMenu::DrawerBackground { drawer } => {
            let d = drawer.clone();
            let d2 = drawer.clone();

            container(
                column![
                    menu_item(
                        "➕  Add apps to this drawer",
                        SearchMessage::OpenAppPicker(d),
                    ),
                    menu_divider(),
                    menu_item(
                        "🗑  Clear all apps from drawer",
                        SearchMessage::ClearDrawer(d2),
                    ),
                ]
                .spacing(2)
            )
            .style(context_menu_style)
            .padding(8)
            .width(Length::Fixed(260.0))
            .into()
        }

        ContextMenu::DrawerApp { drawer, app_id } => {
            let d = drawer.clone();
            let id = app_id.clone();

            container(
                column![
                    menu_item(
                        "✖  Remove from this drawer",
                        SearchMessage::RemoveAppFromDrawer(d, id),
                    ),
                ]
                .spacing(2)
            )
            .style(context_menu_style)
            .padding(8)
            .width(Length::Fixed(260.0))
            .into()
        }
    }
}

fn menu_item<'a>(
    label: &'a str,
    msg: SearchMessage,
) -> Element<'a, SearchMessage> {
    mouse_area(
        container(text(label).size(14))
            .padding([8, 12])
            .width(Length::Fill)
            .style(|_: &Theme| container::Style {
                border: cosmic::iced::border::rounded(4),
                ..Default::default()
            })
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

fn context_menu_style(
    _: &Theme,
) -> container::Style {
    container::Style {
        background: Some(Color::from_rgb8(45, 45, 55).into()),
        border: cosmic::iced::border::rounded(8),
        ..Default::default()
    }
}

// ── App picker view ───────────────────────────────────────────────────────────

fn app_picker_view<'a>(
    search: &'a crate::search::Search,
    picker: &'a crate::search::AppPicker,
) -> Element<'a, SearchMessage> {
    let header = row![
        text(format!("Add apps to \"{}\"", picker.drawer)).size(20),
        space::horizontal().width(Length::Fill),
        mouse_area(
            container(text("✖ Close").size(13)).padding([4, 8])
        )
        .on_press(SearchMessage::CloseAppPicker),
    ]
    .align_y(Vertical::Center)
    .padding([0, 0, 12, 0]);

    let search_bar = text_input(
        "Filter apps...",
        &picker.query,
    )
    .on_input(SearchMessage::AppPickerQueryChanged)
    .padding(12)
    .size(16);

    let app_entries: Vec<_> = picker
        .filtered
        .iter()
        .filter_map(|&i| search.app(i))
        .collect();

    // HashSet for O(1) lookup instead of O(n) per app per frame
    let already_pinned: std::collections::HashSet<&str> = search
        .drawer_state
        .apps_in_drawer(&picker.drawer)
        .iter()
        .map(|s| s.as_str())
        .collect();

    let grid = app_entries
        .chunks(GRID_COLUMNS)
        .fold(column!().spacing(8), |col, chunk| {
            let mut grid_row =
                row!().spacing(8).width(Length::Fill);

            for app in chunk {
                let is_added =
                    already_pinned.contains(app.id.as_str());

                grid_row = grid_row.push(
                    picker_app_icon(app, &picker.drawer, is_added)
                );
            }

            for _ in chunk.len()..GRID_COLUMNS {
                grid_row = grid_row.push(
                    container(
                        space::horizontal().width(Length::Fixed(0.0))
                    )
                    .width(Length::Fill)
                );
            }

            col.push(grid_row)
        });

    container(
        column![
            header,
            container(search_bar).padding([0, 0, 12, 0]),
            scrollable(
                container(grid).padding([0, 12, 0, 0])
            ),
        ]
        .spacing(0)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn picker_app_icon<'a>(
    app: &'a crate::indexer::AppEntry,
    drawer: &str,
    is_added: bool,
) -> Element<'a, SearchMessage> {
    let icon_widget = image(&app.icon_path)
        .width(Length::Fixed(ICON_SIZE))
        .height(Length::Fixed(ICON_SIZE));

    let label = text(truncate_label(&app.name, 12))
        .size(12)
        .center();

    let indicator = if is_added {
        text("✓").size(11)
    } else {
        text("+ Add").size(11)
    };

    let content = column![icon_widget, label, indicator]
        .spacing(2)
        .align_x(Horizontal::Center)
        .width(Length::Fill);

    let msg = if is_added {
        SearchMessage::RemoveAppFromDrawer(
            drawer.to_string(),
            app.id.clone(),
        )
    } else {
        SearchMessage::AddAppToDrawer(
            drawer.to_string(),
            app.id.clone(),
        )
    };

    Element::from(
        mouse_area(
            container(content)
                .padding(6)
                .width(Length::Fill)
                .style(move |_: &Theme| container::Style {
                    background: if is_added {
                        Some(Color::from_rgb8(40, 70, 40).into())
                    } else {
                        None
                    },
                    border: cosmic::iced::border::rounded(6),
                    ..Default::default()
                })
        )
        .on_press(msg)
    )
}

// ── Search results view ───────────────────────────────────────────────────────

fn search_results_view<'a>(
    search: &'a crate::search::Search,
) -> Element<'a, SearchMessage> {
    let results = search.filtered_apps();

    if results.is_empty() {
        return container(text("No apps found").size(16))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
    }

    let app_entries: Vec<_> = results
        .iter()
        .filter_map(|&index| search.app(index))
        .collect();

    let grid = app_entries
        .chunks(GRID_COLUMNS)
        .fold(column!().spacing(8), |col, chunk| {
            let mut grid_row =
                row!().spacing(8).width(Length::Fill);

            for app in chunk {
                grid_row = grid_row.push(app_icon_button(app));
            }

            for _ in chunk.len()..GRID_COLUMNS {
                grid_row = grid_row.push(
                    container(
                        space::horizontal().width(Length::Fixed(0.0))
                    )
                    .width(Length::Fill)
                );
            }

            col.push(grid_row)
        });

    container(scrollable(container(grid).padding([0, 12, 0, 0])))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn app_icon_button<'a>(
    app: &'a crate::indexer::AppEntry,
) -> Element<'a, SearchMessage> {
    let icon_widget = image(&app.icon_path)
        .width(Length::Fixed(ICON_SIZE))
        .height(Length::Fixed(ICON_SIZE));

    let label = text(truncate_label(&app.name, 12))
        .size(12)
        .center();

    let content = column![icon_widget, label]
        .spacing(4)
        .align_x(Horizontal::Center)
        .width(Length::Fill);

    Element::from(
        mouse_area(
            container(content).padding(6).width(Length::Fill)
        )
        .on_press(SearchMessage::AppClicked(app.exec.clone()))
    )
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn truncate_label(name: &str, max_chars: usize) -> String {
    if name.chars().count() <= max_chars {
        return name.to_string();
    }

    let truncated: String = name
        .chars()
        .take(max_chars.saturating_sub(1))
        .collect();

    format!("{truncated}…")
}

// === DONE ===
// Imports back to cosmic::iced::widget to match cosmic::iced::application :: done
// Theme imported as cosmic::iced::Theme via use cosmic::iced::Theme :: done
// All style closures typed as |_: &Theme| explicitly :: done
// context_menu_style takes &Theme (cosmic::iced::Theme) :: done
// HashSet for pinned lookup preserved :: done

// === DONE ===
// Switched all widget imports from cosmic::iced::widget to cosmic::widget :: done
// Element comes from cosmic::prelude::* :: done
// context_menu_style takes &cosmic::Theme not &cosmic::iced::Theme :: done
// style closures use cosmic::iced::widget::container::Style directly :: done
// already_pinned uses HashSet for O(1) lookup per app per frame :: done
// All alignment imports consolidated to cosmic::iced::alignment :: done

// === DONE ===
// Drawer contents now renders real pinned apps :: done
// Right-click on drawer background → context menu (Add apps / Clear) :: done
// Right-click on app in drawer → context menu (Remove) :: done
// App picker modal: filter, add, remove, shows checkmark if already added :: done
// Active drawer highlighted in sidebar :: done
// App count shown next to each drawer name :: done
// Search results view preserved unchanged :: done

// === DONE ===
// Removed rendering-time filesystem icon resolution :: done
// Removed resolve_icon_path entirely :: done
// UI now uses pre-resolved icon_path directly :: done
// Zero filesystem access during typing/rendering :: done
// Faster render path architecture implemented :: done
// Grid layout preserved :: done
// Lightweight mouse_area interactions preserved :: done

// === DONE ===
// Replaced heavy button widgets with lightweight mouse_area :: done
// Direct clickable icons :: done
// Added fallback icon support :: done
// Added icon path resolution :: done
// Removed expensive button styling :: done
// Grid layout preserved :: done
// Faster rendering + snappier scrolling :: done