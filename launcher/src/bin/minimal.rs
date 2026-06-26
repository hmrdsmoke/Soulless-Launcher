// Minimal layer-shell daemon — FLOOD TEST RIG (disposable).
// Replicates cosmic-launcher's surface path EXACTLY: dummy Layer::Bottom surface
// + overlap_notify + handle_overlap + SurfaceState show/hide. Trivial text view.
// Tests whether their exact choreography stops the ~110fps invalidation flood.
// If yes -> apply the pattern to the real launcher (app.rs), delete this.
// Same crate => identical libcosmic dep/features/build.

use cosmic::prelude::*;
use cosmic::app::{Core, Settings};
use cosmic::cctk::sctk::shell::wlr_layer;
use cosmic::iced::event::wayland::OverlapNotifyEvent;
use cosmic::iced::platform_specific::runtime::wayland::layer_surface::SctkLayerSurfaceSettings;
use cosmic::iced::platform_specific::shell::commands::layer_surface::{
    destroy_layer_surface, get_layer_surface,
};
use cosmic::iced::platform_specific::shell::wayland::commands::overlap_notify::overlap_notify;
use cosmic::iced::runtime::core::event::wayland;
use cosmic::iced::runtime::core::event::PlatformSpecific;
use cosmic::iced::runtime::core::layout::Limits;
use cosmic::iced::runtime::platform_specific::wayland::layer_surface::{IcedMargin, IcedOutput};
use cosmic::iced::window;
use cosmic::iced::{Rectangle, Subscription};
use std::collections::HashMap;

fn main() -> cosmic::iced::Result {
    {
        use tracing_subscriber::fmt;
        use tracing_subscriber::EnvFilter;
        let _ = fmt()
            .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")))
            .try_init();
    }
    // EXACT cosmic-launcher run() settings (their src/app.rs run()). Notably NO
    // transparent(), client_decorations(true), explicit text size + scale.
    let settings = Settings::default()
        .antialiasing(true)
        .client_decorations(true)
        .debug(false)
        .default_text_size(16.0)
        .scale_factor(1.0)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceState { Hidden, Visible }

#[derive(Debug, Clone)]
pub enum Message {
    Opened(window::Id),
    Overlap(OverlapNotifyEvent),
}

pub struct Minimal {
    core: Core,
    window_id: window::Id,
    dummy_id: Option<window::Id>,
    state: SurfaceState,
    overlap: HashMap<String, Rectangle>,
    margin: f32,
    height: f32,
}

impl Minimal {
    // EXACT replica of cosmic-launcher's create_dummy_layer_surface (src/app.rs:203).
    fn create_dummy_layer_surface(&mut self) -> Task<cosmic::Action<Message>> {
        let id = window::Id::unique();
        self.dummy_id = Some(id);
        self.handle_overlap();
        Task::batch(vec![
            get_layer_surface(SctkLayerSurfaceSettings {
                id,
                layer: wlr_layer::Layer::Bottom,
                keyboard_interactivity: wlr_layer::KeyboardInteractivity::None,
                input_zone: Some(Vec::new()),
                anchor: wlr_layer::Anchor::TOP,
                output: IcedOutput::Active,
                namespace: "soulless_minimal_dummy".into(),
                margin: IcedMargin::default(),
                size: Some((Some(600), Some(200))),
                exclusive_zone: -1,
                size_limits: Limits::NONE,
            })
            .map(|id| cosmic::Action::App(Message::Opened(id))),
            overlap_notify(id, true).map(|_: ()| cosmic::Action::App(Message::Opened(window::Id::NONE))),
        ])
    }

    fn show(&mut self) -> Task<cosmic::Action<Message>> {
        self.state = SurfaceState::Visible;
        get_layer_surface(SctkLayerSurfaceSettings {
            id: self.window_id,
            keyboard_interactivity: wlr_layer::KeyboardInteractivity::Exclusive,
            anchor: wlr_layer::Anchor::BOTTOM,
            namespace: "soulless-minimal".into(),
            size: Some((Some(700), Some(900))),
            size_limits: Limits::NONE.min_width(1.0).min_height(1.0).max_width(700.0).max_height(900.0),
            exclusive_zone: -1,
            ..Default::default()
        })
        .map(|id| cosmic::Action::App(Message::Opened(id)))
    }

    fn hide(&mut self) -> Task<cosmic::Action<Message>> {
        self.state = SurfaceState::Hidden;
        destroy_layer_surface(self.window_id)
    }

    // EXACT replica of cosmic-launcher's handle_overlap (src/app.rs:281).
    fn handle_overlap(&mut self) {
        let mid_height = self.height / 2.;
        self.margin = 0.;
        for o in self.overlap.values() {
            if self.margin + mid_height < o.y
                || self.margin > o.y + o.height
                || mid_height < o.y + o.height / 2.0
            {
                continue;
            }
            self.margin = o.y + o.height;
        }
    }
}

impl cosmic::Application for Minimal {
    type Executor = cosmic::executor::Default;
    type Flags = MinimalFlags;
    type Message = Message;
    const APP_ID: &'static str = "com.github.hmrdsmoke.SoullessMinimal";

    fn core(&self) -> &Core { &self.core }
    fn core_mut(&mut self) -> &mut Core { &mut self.core }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let mut app = Minimal {
            core,
            window_id: window::Id::unique(),
            dummy_id: None,
            state: SurfaceState::Hidden,
            overlap: HashMap::new(),
            margin: 0.0,
            height: 0.0,
        };
        let task = app.create_dummy_layer_surface();
        (app, task)
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::Overlap(ev) => {
                match ev {
                    OverlapNotifyEvent::OverlapLayerAdd { identifier, logical_rect, .. } => {
                        self.overlap.insert(identifier, logical_rect);
                        self.handle_overlap();
                    }
                    OverlapNotifyEvent::OverlapLayerRemove { identifier } => {
                        self.overlap.remove(&identifier);
                        self.handle_overlap();
                    }
                    _ => {}
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn dbus_activation(
        &mut self,
        msg: cosmic::dbus_activation::Message,
    ) -> Task<cosmic::Action<Self::Message>> {
        use cosmic::dbus_activation::Details;
        eprintln!("[DBUS] activation received");
        match msg.msg {
            Details::Activate => {
                if self.state == SurfaceState::Visible {
                    eprintln!("[DBUS] Activate -> hide()");
                    self.hide()
                } else {
                    eprintln!("[DBUS] Activate -> show()");
                    self.show()
                }
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
            // overlap events arrive via the wayland event stream.
            // Return None for everything else — emitting a message per frame
            // event (the old `other => Some(...)` catch-all) re-arms the redraw
            // loop and is the flood.
            cosmic::iced::event::listen_with(|e, _status, _id| match e {
                cosmic::iced::Event::PlatformSpecific(PlatformSpecific::Wayland(
                    wayland::Event::OverlapNotify(event, ..),
                )) => Some(Message::Overlap(event)),
                _ => None,
            }),
        ])
    }
}
