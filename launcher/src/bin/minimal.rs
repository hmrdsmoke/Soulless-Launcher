// Minimal layer-shell daemon — isolation test for the RequestResize flood.
// Same crate as the launcher => identical libcosmic dep/features/build.
// Strips EVERYTHING (no Themer/autosize/monitors/search). Uses the EXACT same
// imports/types as the real app.rs/placement.rs so trait signatures match.

use cosmic::prelude::*;
use cosmic::app::{Core, Settings};
use cosmic::iced::advanced::layout::Limits;
use cosmic::iced::platform_specific::shell::commands::layer_surface::{
    get_layer_surface, Anchor, KeyboardInteractivity, Layer,
};
use cosmic::iced::platform_specific::runtime::wayland::layer_surface::SctkLayerSurfaceSettings;
use cosmic::iced::window;
use cosmic::iced::Subscription;

fn main() -> cosmic::iced::Result {
    {
        use tracing_subscriber::fmt;
        use tracing_subscriber::EnvFilter;
        let _ = fmt()
            .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")))
            .try_init();
    }
    let settings = Settings::default()
        .client_decorations(false)
        .transparent(true)
        .resizable(None)
        .no_main_window(true)
        .exit_on_close(false);
    cosmic::app::run_single_instance::<Minimal>(settings, MinimalFlags::default())
}

#[derive(Debug, Clone)]
pub enum MinimalSub { Toggle }
impl std::fmt::Display for MinimalSub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "Toggle") }
}
#[derive(Debug, Clone, Default)]
pub struct MinimalFlags { pub subcommand: Option<MinimalSub> }
impl cosmic::app::CosmicFlags for MinimalFlags {
    type SubCommand = MinimalSub;
    type Args = Vec<String>;
    fn action(&self) -> Option<&Self::SubCommand> { self.subcommand.as_ref() }
}

#[derive(Debug, Clone)]
pub enum Message {
    Opened(window::Id),
    Event(cosmic::iced::Event),
    Noop,
}

pub struct Minimal {
    core: Core,
    window_id: window::Id,
}

impl cosmic::Application for Minimal {
    type Executor = cosmic::executor::Default;
    type Flags = MinimalFlags;
    type Message = Message;
    const APP_ID: &'static str = "com.github.hmrdsmoke.SoullessMinimal";

    fn core(&self) -> &Core { &self.core }
    fn core_mut(&mut self) -> &mut Core { &mut self.core }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let window_id = window::Id::unique();
        (Minimal { core, window_id }, Task::none())
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        if let Message::Event(cosmic::iced::Event::PlatformSpecific(
            cosmic::iced::event::PlatformSpecific::Wayland(we),
        )) = &message {
            eprintln!("[EVT] Wayland: {:?}", we);
        }
        Task::none()
    }

    fn dbus_activation(
        &mut self,
        msg: cosmic::dbus_activation::Message,
    ) -> Task<cosmic::Action<Self::Message>> {
        use cosmic::dbus_activation::Details;
        eprintln!("[DBUS] activation received");
        match msg.msg {
            Details::Activate => {
                eprintln!("[DBUS] Activate -> creating minimal surface");
                let mut s = SctkLayerSurfaceSettings::default();
                s.id = self.window_id;
                s.keyboard_interactivity = KeyboardInteractivity::Exclusive;
                s.layer = Layer::Top;
                s.anchor = Anchor::BOTTOM;
                s.size = Some((Some(700), Some(900)));
                s.size_limits = Limits::NONE.min_width(1.0).min_height(1.0).max_width(700.0).max_height(900.0);
                s.exclusive_zone = -1;
                s.namespace = "soulless-minimal".to_string();
                get_layer_surface(s).map(|id| cosmic::Action::App(Message::Opened(id)))
            }
            _ => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        cosmic::widget::text("minimal").into()
    }

    fn view_window(&self, _id: window::Id) -> Element<'_, Self::Message> {
        cosmic::widget::text("minimal").into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch([
            cosmic::iced::Subscription::run_with("minimal-keepalive", |_| {
                cosmic::iced::stream::channel(1, |_s| async move {
                    std::future::pending::<()>().await;
                })
            }),
            cosmic::iced::event::listen().map(Message::Event),
        ])
    }
}