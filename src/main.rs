use cosmic::app::{Core, Settings, Task};
use cosmic::cctk::sctk::shell::wlr_layer::Layer;
use cosmic::iced::platform_specific::runtime::wayland::layer_surface::{
    IcedMargin, IcedOutput, SctkLayerSurfaceSettings,
};
use cosmic::iced::platform_specific::shell::commands::layer_surface::{
    self as layer_cmd, KeyboardInteractivity,
};
use cosmic::iced::runtime::core::layout::Limits;
use cosmic::iced::window::Id as SurfaceId;
use cosmic::iced::{Event, Subscription};
use cosmic::{Application, Element};
use tracing::info;

use std::sync::Arc;
use toodle::config::Config;
use toodle::popup::{self, PopupMessage};
use toodle::weather::{
    weather_update_stream, WeatherCache, WeatherData, WeatherError, WeatherService,
};
use toodle::widget::edit_mode::{EditMessage, EditState};
use toodle::widget::{self, WidgetMessage, WidgetState};

#[derive(Debug, Clone)]
enum ActivePopup {
    ContextMenu,
    Calendar(popup::CalendarState),
    Forecast,
}

struct ToodleApp {
    core: Core,
    config: Config,
    state: WidgetState,
    weather_service: Arc<WeatherService>,
    weather: Option<WeatherData>,
    weather_error: bool,
    widget_surface_id: SurfaceId,
    popup_surface_id: Option<SurfaceId>,
    active_popup: Option<ActivePopup>,
    edit_panel_surface_id: Option<SurfaceId>,
}

#[derive(Debug, Clone)]
enum Message {
    Widget(WidgetMessage),
    Popup(PopupMessage),
    Edit(EditMessage),
    Tick,
    EscapePressed,
    ConfigReloaded(Config),
    FetchWeather,
    WeatherUpdated(Result<WeatherData, WeatherError>),
}

impl From<WidgetMessage> for Message {
    fn from(msg: WidgetMessage) -> Self {
        Message::Widget(msg)
    }
}

impl From<PopupMessage> for Message {
    fn from(msg: PopupMessage) -> Self {
        Message::Popup(msg)
    }
}

impl From<EditMessage> for Message {
    fn from(msg: EditMessage) -> Self {
        Message::Edit(msg)
    }
}

impl Application for ToodleApp {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "com.github.wammed.toodle";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let config = Config::load();
        let widget_surface_id = SurfaceId::unique();

        let (top, right, bottom, left) = config.layout.margins();
        let widget_settings = SctkLayerSurfaceSettings {
            id: widget_surface_id,
            layer: Layer::Bottom,
            keyboard_interactivity: KeyboardInteractivity::None,
            input_zone: Some(widget::content_bounds(&config)),
            anchor: config.layout.anchor.to_layer_anchor(),
            output: IcedOutput::Active,
            namespace: "toodle-widget".to_string(),
            margin: IcedMargin {
                top,
                right,
                bottom,
                left,
            },
            size: Some((Some(config.layout.width), Some(config.layout.height))),
            exclusive_zone: 0,
            size_limits: Limits::NONE,
        };

        let create_widget_task = layer_cmd::get_layer_surface(widget_settings);

        // Load embedded fonts on startup
        let load_fonts_tasks: Vec<_> = toodle::clock::fonts::embedded_fonts()
            .into_iter()
            .map(|bytes| cosmic::iced::font::load(bytes).discard())
            .collect();

        let weather_service = Arc::new(WeatherService::new());
        let cached_weather = WeatherCache::load().map(|c| c.data);

        let initial_weather_task = {
            let s = weather_service.clone();
            let lat = config.weather.latitude;
            let lon = config.weather.longitude;
            Task::future(async move {
                let res = s.get_weather(lat, lon).await;
                cosmic::Action::from(Message::WeatherUpdated(res))
            })
        };

        let initial_tasks = Task::batch(
            std::iter::once(create_widget_task)
                .chain(load_fonts_tasks)
                .chain(std::iter::once(initial_weather_task)),
        );

        let app = Self {
            core,
            config,
            state: WidgetState::Normal,
            weather_service,
            weather: cached_weather,
            weather_error: false,
            widget_surface_id,
            popup_surface_id: None,
            active_popup: None,
            edit_panel_surface_id: None,
        };

        (app, initial_tasks)
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Tick => Task::none(),

            Message::FetchWeather => {
                let s = self.weather_service.clone();
                let lat = self.config.weather.latitude;
                let lon = self.config.weather.longitude;
                Task::future(async move {
                    let res = s.get_weather(lat, lon).await;
                    cosmic::Action::from(Message::WeatherUpdated(res))
                })
            }

            Message::WeatherUpdated(res) => {
                match res {
                    Ok(data) => {
                        info!(
                            "Weather updated: {} {:.1}°C",
                            data.current.condition_text, data.current.temperature_celsius
                        );
                        self.weather = Some(data);
                        self.weather_error = false;
                    }
                    Err(err) => {
                        tracing::warn!("Weather update failed: {}", err);
                        if self.weather.is_none() {
                            self.weather_error = true;
                        }
                    }
                }
                Task::none()
            }

            Message::Widget(WidgetMessage::RightClicked) => {
                let mut tasks = Vec::new();

                // If popup already open, destroy it first
                if let Some(existing_popup) = self.popup_surface_id.take() {
                    tasks.push(layer_cmd::destroy_layer_surface(existing_popup));
                }

                let new_popup_id = SurfaceId::unique();
                self.popup_surface_id = Some(new_popup_id);
                self.active_popup = Some(ActivePopup::ContextMenu);

                let (w_top, w_right, w_bottom, w_left) = self.config.layout.margins();
                let popup_margin = IcedMargin {
                    top: w_top + 16,
                    right: w_right + 16,
                    bottom: w_bottom + 16,
                    left: w_left + 16,
                };

                let popup_settings = SctkLayerSurfaceSettings {
                    id: new_popup_id,
                    layer: Layer::Top,
                    keyboard_interactivity: KeyboardInteractivity::OnDemand,
                    input_zone: None,
                    anchor: self.config.layout.anchor.to_layer_anchor(),
                    output: IcedOutput::Active,
                    namespace: "toodle-popup".to_string(),
                    margin: popup_margin,
                    size: Some((Some(340), Some(380))),
                    exclusive_zone: 0,
                    size_limits: Limits::NONE,
                };

                tasks.push(layer_cmd::get_layer_surface(popup_settings));
                Task::batch(tasks)
            }

            Message::Popup(PopupMessage::EditLayout) => {
                let mut tasks = Vec::new();
                if let Some(popup_id) = self.popup_surface_id.take() {
                    tasks.push(layer_cmd::destroy_layer_surface(popup_id));
                }
                self.active_popup = None;

                // Initialize Edit State with current layout and font_scale
                self.state = WidgetState::Edit(EditState::new(
                    self.config.layout.clone(),
                    self.config.appearance.font_scale,
                ));

                // Open independent Edit Panel on Layer::Top
                let edit_panel_id = SurfaceId::unique();
                self.edit_panel_surface_id = Some(edit_panel_id);

                // Place the edit panel at an accessible position on screen
                let panel_margin = IcedMargin {
                    top: 60,
                    right: 60,
                    bottom: 0,
                    left: 0,
                };

                use cosmic::iced::platform_specific::shell::commands::layer_surface::Anchor as LayerAnchor;
                let panel_settings = SctkLayerSurfaceSettings {
                    id: edit_panel_id,
                    layer: Layer::Top,
                    keyboard_interactivity: KeyboardInteractivity::OnDemand,
                    input_zone: None,
                    anchor: LayerAnchor::TOP | LayerAnchor::RIGHT,
                    output: IcedOutput::Active,
                    namespace: "toodle-edit-panel".to_string(),
                    margin: panel_margin,
                    size: Some((Some(420), Some(480))),
                    exclusive_zone: 0,
                    size_limits: Limits::NONE,
                };

                tasks.push(layer_cmd::get_layer_surface(panel_settings));
                // Set entire widget surface to accept input
                tasks.push(layer_cmd::set_input_zone(self.widget_surface_id, None));
                Task::batch(tasks)
            }

            Message::Popup(PopupMessage::OpenSettings) => {
                let mut tasks = Vec::new();
                if let Some(popup_id) = self.popup_surface_id.take() {
                    tasks.push(layer_cmd::destroy_layer_surface(popup_id));
                }
                self.active_popup = None;

                let _ = std::process::Command::new("toodle-settings").spawn();
                Task::batch(tasks)
            }

            Message::Popup(PopupMessage::OpenCalendar) => {
                let mut tasks = Vec::new();
                if let Some(popup_id) = self.popup_surface_id.take() {
                    tasks.push(layer_cmd::destroy_layer_surface(popup_id));
                }

                let new_popup_id = SurfaceId::unique();
                self.popup_surface_id = Some(new_popup_id);
                self.active_popup = Some(ActivePopup::Calendar(popup::CalendarState::new()));

                let (w_top, w_right, w_bottom, w_left) = self.config.layout.margins();
                let popup_margin = IcedMargin {
                    top: w_top + 16,
                    right: w_right + 16,
                    bottom: w_bottom + 16,
                    left: w_left + 16,
                };

                let popup_settings = SctkLayerSurfaceSettings {
                    id: new_popup_id,
                    layer: Layer::Top,
                    keyboard_interactivity: KeyboardInteractivity::OnDemand,
                    input_zone: None,
                    anchor: self.config.layout.anchor.to_layer_anchor(),
                    output: IcedOutput::Active,
                    namespace: "toodle-calendar".to_string(),
                    margin: popup_margin,
                    size: Some((Some(680), Some(720))),
                    exclusive_zone: 0,
                    size_limits: Limits::NONE,
                };

                tasks.push(layer_cmd::get_layer_surface(popup_settings));
                Task::batch(tasks)
            }

            Message::Popup(PopupMessage::OpenForecast) => {
                let mut tasks = Vec::new();
                if let Some(popup_id) = self.popup_surface_id.take() {
                    tasks.push(layer_cmd::destroy_layer_surface(popup_id));
                }

                let new_popup_id = SurfaceId::unique();
                self.popup_surface_id = Some(new_popup_id);
                self.active_popup = Some(ActivePopup::Forecast);

                let (w_top, w_right, w_bottom, w_left) = self.config.layout.margins();
                let popup_margin = IcedMargin {
                    top: w_top + 16,
                    right: w_right + 16,
                    bottom: w_bottom + 16,
                    left: w_left + 16,
                };

                let popup_settings = SctkLayerSurfaceSettings {
                    id: new_popup_id,
                    layer: Layer::Top,
                    keyboard_interactivity: KeyboardInteractivity::OnDemand,
                    input_zone: None,
                    anchor: self.config.layout.anchor.to_layer_anchor(),
                    output: IcedOutput::Active,
                    namespace: "toodle-forecast".to_string(),
                    margin: popup_margin,
                    size: Some((Some(680), Some(720))),
                    exclusive_zone: 0,
                    size_limits: Limits::NONE,
                };

                tasks.push(layer_cmd::get_layer_surface(popup_settings));
                Task::batch(tasks)
            }

            Message::Popup(PopupMessage::CalendarPrevMonth) => {
                if let Some(ActivePopup::Calendar(ref mut state)) = self.active_popup {
                    state.prev_month();
                }
                Task::none()
            }

            Message::Popup(PopupMessage::CalendarNextMonth) => {
                if let Some(ActivePopup::Calendar(ref mut state)) = self.active_popup {
                    state.next_month();
                }
                Task::none()
            }

            Message::Popup(PopupMessage::CalendarToday) => {
                if let Some(ActivePopup::Calendar(ref mut state)) = self.active_popup {
                    state.jump_today();
                }
                Task::none()
            }

            Message::Popup(PopupMessage::Close) => {
                self.active_popup = None;
                if let Some(popup_id) = self.popup_surface_id.take() {
                    layer_cmd::destroy_layer_surface(popup_id)
                } else {
                    Task::none()
                }
            }

            Message::Popup(PopupMessage::Quit) => {
                std::process::exit(0);
            }

            Message::Edit(EditMessage::Save) => {
                let mut tasks = Vec::new();
                if let WidgetState::Edit(edit_state) = &self.state {
                    self.config.layout = edit_state.layout.clone();
                    self.config.appearance.font_scale = edit_state.font_scale;
                    let _ = self.config.save();
                }

                if let Some(panel_id) = self.edit_panel_surface_id.take() {
                    tasks.push(layer_cmd::destroy_layer_surface(panel_id));
                }

                self.state = WidgetState::Normal;

                // The widget surface is ALREADY at the exact desired anchor, size, and margin!
                // We do NOT need to call set_anchor / set_margin / set_size again.
                // Doing so forces Wayland buffer reallocations which caused the flickering!
                // We only need to restore the input zone to content bounds.
                tasks.push(layer_cmd::set_input_zone(
                    self.widget_surface_id,
                    Some(widget::content_bounds(&self.config)),
                ));

                Task::batch(tasks)
            }

            Message::Edit(EditMessage::Cancel) | Message::EscapePressed => {
                let mut tasks = Vec::new();
                if let Some(panel_id) = self.edit_panel_surface_id.take() {
                    tasks.push(layer_cmd::destroy_layer_surface(panel_id));
                }
                if let Some(popup_id) = self.popup_surface_id.take() {
                    tasks.push(layer_cmd::destroy_layer_surface(popup_id));
                }
                self.active_popup = None;

                self.state = WidgetState::Normal;
                let (top, right, bottom, left) = self.config.layout.margins();

                tasks.push(layer_cmd::set_anchor(
                    self.widget_surface_id,
                    self.config.layout.anchor.to_layer_anchor(),
                ));
                tasks.push(layer_cmd::set_margin(
                    self.widget_surface_id,
                    top,
                    right,
                    bottom,
                    left,
                ));
                tasks.push(layer_cmd::set_size(
                    self.widget_surface_id,
                    Some(self.config.layout.width),
                    Some(self.config.layout.height),
                ));
                tasks.push(layer_cmd::set_input_zone(
                    self.widget_surface_id,
                    Some(widget::content_bounds(&self.config)),
                ));

                Task::batch(tasks)
            }

            Message::Edit(msg) => {
                if let WidgetState::Edit(edit_state) = &mut self.state {
                    match msg {
                        EditMessage::SetMarginX(_) | EditMessage::SetMarginY(_) => {
                            edit_state.update(msg);
                            let (top, right, bottom, left) = edit_state.layout.margins();
                            layer_cmd::set_margin(self.widget_surface_id, top, right, bottom, left)
                        }
                        EditMessage::SetAnchor(_) => {
                            edit_state.update(msg);
                            let (top, right, bottom, left) = edit_state.layout.margins();
                            let anchor = edit_state.layout.anchor.to_layer_anchor();
                            Task::batch(vec![
                                layer_cmd::set_anchor(self.widget_surface_id, anchor),
                                layer_cmd::set_margin(self.widget_surface_id, top, right, bottom, left),
                            ])
                        }
                        EditMessage::SetWidth(_) | EditMessage::SetHeight(_) => {
                            edit_state.update(msg);
                            layer_cmd::set_size(
                                self.widget_surface_id,
                                Some(edit_state.layout.width),
                                Some(edit_state.layout.height),
                            )
                        }
                        EditMessage::SetFontScale(_) => {
                            edit_state.update(msg);
                            Task::none()
                        }
                        EditMessage::Save | EditMessage::Cancel => Task::none(),
                    }
                } else {
                    Task::none()
                }
            }

            Message::ConfigReloaded(new_config) => {
                // If config has not changed (e.g. triggered by our own Save), do nothing!
                if self.config == new_config {
                    return Task::none();
                }

                info!("Config reloaded via file watcher from external change");
                let location_changed = self.config.weather.latitude != new_config.weather.latitude
                    || self.config.weather.longitude != new_config.weather.longitude;

                self.config = new_config;

                let mut tasks = Vec::new();

                if location_changed {
                    let s = self.weather_service.clone();
                    let lat = self.config.weather.latitude;
                    let lon = self.config.weather.longitude;
                    tasks.push(Task::future(async move {
                        let res = s.get_weather(lat, lon).await;
                        cosmic::Action::from(Message::WeatherUpdated(res))
                    }));
                }

                if let WidgetState::Normal = self.state {
                    let (top, right, bottom, left) = self.config.layout.margins();
                    tasks.push(layer_cmd::set_anchor(
                        self.widget_surface_id,
                        self.config.layout.anchor.to_layer_anchor(),
                    ));
                    tasks.push(layer_cmd::set_margin(self.widget_surface_id, top, right, bottom, left));
                    tasks.push(layer_cmd::set_size(
                        self.widget_surface_id,
                        Some(self.config.layout.width),
                        Some(self.config.layout.height),
                    ));
                    tasks.push(layer_cmd::set_input_zone(
                        self.widget_surface_id,
                        Some(widget::content_bounds(&self.config)),
                    ));
                }

                Task::batch(tasks)
            }
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let tick = cosmic::iced::Subscription::run(toodle::clock::next_second_tick)
            .map(|_| Message::Tick);
        let watcher = cosmic::iced::Subscription::run(Config::watch).map(Message::ConfigReloaded);
        let weather_sub = cosmic::iced::Subscription::run(weather_update_stream)
            .map(|_| Message::FetchWeather);

        // Filter events strictly to avoid flooding the message queue with cursor movements!
        let escape_key = cosmic::iced::event::listen_with(|event, _status, _id| {
            if let Event::Keyboard(cosmic::iced::keyboard::Event::KeyPressed {
                key: cosmic::iced::keyboard::Key::Named(cosmic::iced::keyboard::key::Named::Escape),
                ..
            }) = event
            {
                Some(Message::EscapePressed)
            } else {
                None
            }
        });

        Subscription::batch(vec![tick, watcher, weather_sub, escape_key])
    }

    fn view(&self) -> Element<'_, Self::Message> {
        cosmic::widget::text("").into()
    }

    fn view_window(&self, id: SurfaceId) -> Element<'_, Self::Message> {
        if id == self.widget_surface_id {
            widget::view_widget(
                &self.state,
                &self.config,
                self.weather.as_ref(),
                self.weather_error,
            )
            .map(Message::Widget)
        } else if Some(id) == self.popup_surface_id {
            match &self.active_popup {
                Some(ActivePopup::ContextMenu) => popup::view_context_menu().map(Message::Popup),
                Some(ActivePopup::Calendar(cal_state)) => {
                    popup::view_calendar(cal_state).map(Message::Popup)
                }
                Some(ActivePopup::Forecast) => {
                    popup::view_forecast(&self.config, self.weather.as_ref()).map(Message::Popup)
                }
                None => cosmic::widget::text("").into(),
            }
        } else if Some(id) == self.edit_panel_surface_id {
            if let WidgetState::Edit(edit_state) = &self.state {
                edit_state.view().map(Message::Edit)
            } else {
                cosmic::widget::text("").into()
            }
        } else {
            cosmic::widget::text("").into()
        }
    }
}

fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt::init();
    info!("Starting toodle (COSMIC Desktop Clock Widget)...");

    cosmic::app::run::<ToodleApp>(
        Settings::default()
            .no_main_window(true)
            .exit_on_close(false)
            .antialiasing(true),
        (),
    )
}
