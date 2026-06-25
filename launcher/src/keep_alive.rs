//! Daemon keep-alive subscription.
//!
//! A `run_single_instance` + `no_main_window` daemon exits when nothing keeps the
//! iced event loop busy. A timer subscription is NOT sufficient — iced treats a
//! windowless daemon as exit-worthy. cosmic-launcher stays resident via a
//! persistent `stream::channel` whose async task never completes (its pop-launcher
//! backend loop). Soulless has no separate backend, so this is the minimal
//! equivalent: an infinite stream that never yields and never returns, holding the
//! daemon open between D-Bus activations.
//!
//! (Daemon-persistence mechanism learned from pop-os/cosmic-launcher, GPL-3.0.)

use crate::app::Message;
use cosmic::iced::stream;

pub fn subscription() -> cosmic::iced::Subscription<Message> {
    cosmic::iced::Subscription::run_with("soulless-keep-alive", |_| {
        stream::channel(1, |_sender| async move {
            // Never completes → iced keeps the daemon's event loop alive.
            // We never send anything on the channel; we just park forever.
            std::future::pending::<()>().await;
        })
    })
}
