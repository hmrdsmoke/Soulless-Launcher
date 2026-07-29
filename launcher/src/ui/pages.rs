// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/ui/pages.rs
//
// ── Widget pages ──────────────────────────────────────────────────────────────
//
// The widget region below the steel toolbox is swappable real estate: a page
// is a curated arrangement filling the same footprint, and the dot strip —
// sitting in the black band between the steel and the widgets — selects one
// directly. Yellow dot = the page you're on. Black dot = a page you're not.
// Click a black dot to jump straight there; no cycling.
//
// Pages are ambient preference during a session (switching preserves all
// widget state), but dismiss snaps home: dbus_activation's fresh-on-open
// reset returns the launcher to Monitors, same as search clearing and the
// vault locking. The launcher always opens looking the same.

use cosmic::iced::widget::{button, container, row, space};
use cosmic::iced::{Color, Element, Length};

/// Every page the dot strip can select. Adding a page = a variant here, a
/// dot (ALL below), and a match arm in app.rs's view().
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    /// Home base: the 2×2 monitor grid. Where every summon starts.
    Monitors,
    /// The scratch command box, wearing the whole 2×2 footprint.
    Terminal,
}

/// Draw order of the dots, left to right.
const ALL: [Page; 2] = [Page::Monitors, Page::Terminal];

const DOT_SIZE: f32 = 10.0;
const DOT_SPACING: f32 = 10.0;

/// Brand yellow — the "you are here" dot. Same family as the terminal
/// box's prompt color; deliberately a local const like the monitors'
/// graph colors (functional signalling, not themed).
const DOT_ACTIVE: Color = Color::from_rgb(1.0, 0.78, 0.16);

/// Inactive dot: black fill with a dim ring so it reads as an empty
/// socket against the near-black window background instead of vanishing.
const DOT_INACTIVE: Color = Color::from_rgb(0.07, 0.07, 0.09);
const DOT_RING: Color = Color::from_rgb(0.38, 0.38, 0.42);

/// The dot strip. Lives in the left column between the steel toolbox and
/// the page area. `on_select` is the app's message constructor
/// (Message::PageSelected) — clicking any dot sends its page; clicking the
/// active dot is a harmless no-op reselect.
pub fn dot_strip<'a, M: Clone + 'static>(
    active: Page,
    on_select: impl Fn(Page) -> M,
) -> Element<'a, M> {
    let mut dots = row![].spacing(DOT_SPACING);

    for page in ALL {
        let is_active = page == active;
        let dot = button(space::horizontal().width(Length::Fixed(0.0)))
            .width(Length::Fixed(DOT_SIZE))
            .height(Length::Fixed(DOT_SIZE))
            .padding(0)
            .on_press(on_select(page))
            .style(move |_theme, _status| cosmic::iced::widget::button::Style {
                background: Some(if is_active { DOT_ACTIVE } else { DOT_INACTIVE }.into()),
                text_color: Color::TRANSPARENT,
                border: cosmic::iced::Border {
                    radius: cosmic::iced::border::rounded(DOT_SIZE / 2.0).radius,
                    color: if is_active { DOT_ACTIVE } else { DOT_RING },
                    width: 1.0,
                },
                ..Default::default()
            });
        dots = dots.push(dot);
    }

    container(dots)
        .width(Length::Fill)
        .align_x(cosmic::iced::alignment::Horizontal::Center)
        .into()
}
