// GPL-3.0-or-later - see LICENSE file for full terms
// Copyright 2026 Michael Van Auker (HMRDSmoke)
// This is my original work with contributions from Claude (Anthropic).
// Do not remove these comments.
// launcher/src/ui/mod.rs
// Launcher UI module: startup focus tasks, region blur task, submodule re-exports.

pub mod theme;
pub mod panels;
pub mod widgets;

/// Focus tasks — run once when the real launcher window opens.
pub fn startup_tasks<Message: 'static + Send>(
    id: cosmic::iced::window::Id,
) -> cosmic::iced::Task<Message> {
    cosmic::iced::Task::batch([
        cosmic::iced::window::gain_focus(id),
        cosmic::widget::text_input::focus(cosmic::widget::Id::new("soulless-search-bar")),
    ])
}

/// Region-scoped compositor blur for the launcher zone. Sent on the surface's
/// own configure (Opened/Resized) with the compositor-reported size, so the
/// region is correct per-output. enable_blur() can't do this — it hardcodes a
/// whole-surface rectangle — so we send the underlying BlurSurface action,
/// same dispatch pattern as the layer_surface commands. The backend updates
/// the region in place on repeat sends.
pub fn blur_task<Message: 'static + Send>(
    id: cosmic::iced::window::Id,
    zone: cosmic::iced::Rectangle,
) -> cosmic::iced::Task<Message> {
    use cosmic::iced::platform_specific::runtime as ps;
    use cosmic::iced::runtime::{self as iced_runtime, task};

    task::effect(iced_runtime::Action::PlatformSpecific(
        ps::Action::Wayland(ps::wayland::Action::BlurSurface(
            id,
            Some(vec![zone]),
        )),
    ))
}
pub mod organizer;