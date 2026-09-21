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
use cosmic::cctk::sctk::reexports::client::protocol::wl_output::WlOutput;
use cosmic::iced::core::event::wayland::{Event as WaylandEvent, OutputEvent};
use cosmic::iced::core::event::PlatformSpecific;
use cosmic::iced::{Event, Subscription};
use cosmic::{Application, Element};
use tracing::info;

use std::sync::Arc;
use toodle::config::Config;
use toodle::display::clean_display_name;
use toodle::popup::{self, PopupMessage};
use toodle::weather::{
    WeatherCache, WeatherData, WeatherError, WeatherService, weather_update_stream,
};
use toodle::widget::edit_mode::{EditMessage, EditState};
use toodle::widget::{self, WidgetMessage, WidgetState};

#[derive(Debug, Clone)]
enum ActivePopup {
    ContextMenu,
    Calendar(popup::CalendarState),
    Forecast,
}

#[derive(Debug, Clone)]
struct OutputEntry {
    name: Option<String>,
    output: WlOutput,
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
    outputs: Vec<OutputEntry>,
    current_assigned_output: Option<String>,
    screen_w: u32,
    screen_h: u32,
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
    WaylandOutput(OutputEvent, WlOutput),
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

impl ToodleApp {
    fn get_target_iced_output(&self) -> IcedOutput {
        if let Some(target) = &self.config.display.output {
            let clean_target = clean_display_name(target);
            if !clean_target.is_empty() {
                for entry in &self.outputs {
                    if let Some(name) = &entry.name {
                        let clean_entry_name = clean_display_name(name);
                        if clean_entry_name.eq_ignore_ascii_case(&clean_target) {
                            return IcedOutput::Output(entry.output.clone());
                        }
                    }
                }
            }
        }
        IcedOutput::Active
    }

    fn recreate_widget(&mut self) -> Task<Message> {
        let old_id = self.widget_surface_id;
        let new_id = SurfaceId::unique();
        self.widget_surface_id = new_id;
        self.current_assigned_output = self.config.display.output.clone();

        let (anchor, (top, right, bottom, left), (w, h), _scale) =
            self.config.layout.calculate_geometry(self.screen_w, self.screen_h);
        let target_output = self.get_target_iced_output();

        let widget_settings = SctkLayerSurfaceSettings {
            id: new_id,
            layer: Layer::Bottom,
            keyboard_interactivity: KeyboardInteractivity::None,
            input_zone: Some(widget::content_bounds(&self.config)),
            anchor,
            output: target_output,
            namespace: "toodle-widget".to_string(),
            margin: IcedMargin {
                top,
                right,
                bottom,
                left,
            },
            size: Some((Some(w), Some(h))),
            exclusive_zone: 0,
            size_limits: Limits::NONE,
        };

        info!(
            "Recreating widget layer surface on target output {:?}",
            self.config.display.output
        );

        let mut tasks = vec![
            layer_cmd::destroy_layer_surface(old_id),
            layer_cmd::get_layer_surface(widget_settings),
        ];

        if let Some(popup_id) = self.popup_surface_id.take() {
            tasks.push(layer_cmd::destroy_layer_surface(popup_id));
            self.active_popup = None;
        }
        if let Some(panel_id) = self.edit_panel_surface_id.take() {
            tasks.push(layer_cmd::destroy_layer_surface(panel_id));
            self.state = WidgetState::Normal;
        }

        Task::batch(tasks)
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

    fn init(mut core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        core.set_auto_blur(Default::default());
        core.set_auto_corner_radius(Default::default());
        let config = Config::load();
        let widget_surface_id = SurfaceId::unique();

        let (screen_w, screen_h) = toodle::display::detect_primary_resolution();
        let (anchor, (top, right, bottom, left), (w, h), _scale) =
            config.layout.calculate_geometry(screen_w, screen_h);
        let widget_settings = SctkLayerSurfaceSettings {
            id: widget_surface_id,
            layer: Layer::Bottom,
            keyboard_interactivity: KeyboardInteractivity::None,
            input_zone: Some(widget::content_bounds(&config)),
            anchor,
            output: IcedOutput::Active,
            namespace: "toodle-widget".to_string(),
            margin: IcedMargin {
                top,
                right,
                bottom,
                left,
            },
            size: Some((Some(w), Some(h))),
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
            outputs: Vec::new(),
            current_assigned_output: None,
            screen_w,
            screen_h,
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
                    output: self.get_target_iced_output(),
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

                // Initialize Edit State with current layout, font_scale, and screen size
                self.state = WidgetState::Edit(EditState::new(
                    self.config.layout.clone(),
                    self.config.appearance.font_scale,
                    self.screen_w,
                    self.screen_h,
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
                    output: self.get_target_iced_output(),
                    namespace: "toodle-edit-panel".to_string(),
                    margin: panel_margin,
                    size: Some((Some(460), Some(490))),
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
                let spawn_res = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|dir| dir.join("toodle-settings")))
                    .filter(|p| p.exists())
                    .map(|exe| std::process::Command::new(exe).spawn())
                    .unwrap_or_else(|| std::process::Command::new("toodle-settings").spawn());

                if let Err(e) = spawn_res {
                    tracing::warn!("Failed to launch toodle-settings: {}", e);
                }

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
                    output: self.get_target_iced_output(),
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
                    output: self.get_target_iced_output(),
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
                    let (_anchor, _margins, (w, h), _) =
                        self.config.layout.calculate_geometry(self.screen_w, self.screen_h);
                    self.config.layout.width = w;
                    self.config.layout.height = h;
                    self.config.layout.anchor = edit_state.layout.grid_position.to_legacy_anchor();
                    let _ = self.config.save();
                }

                if let Some(panel_id) = self.edit_panel_surface_id.take() {
                    tasks.push(layer_cmd::destroy_layer_surface(panel_id));
                }

                self.state = WidgetState::Normal;

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
                let (anchor, (top, right, bottom, left), (w, h), _) =
                    self.config.layout.calculate_geometry(self.screen_w, self.screen_h);

                tasks.push(layer_cmd::set_anchor(
                    self.widget_surface_id,
                    anchor,
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
                    Some(w),
                    Some(h),
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
                        EditMessage::SetGridPosition(pos) => {
                            edit_state.update(EditMessage::SetGridPosition(pos));
                            let (anchor, (top, right, bottom, left), _size, _scale) =
                                edit_state.layout.calculate_geometry(self.screen_w, self.screen_h);
                            Task::batch(vec![
                                layer_cmd::set_anchor(self.widget_surface_id, anchor),
                                layer_cmd::set_margin(
                                    self.widget_surface_id,
                                    top,
                                    right,
                                    bottom,
                                    left,
                                ),
                            ])
                        }
                        EditMessage::SetSizeStage(stage) => {
                            edit_state.update(EditMessage::SetSizeStage(stage));
                            let (anchor, (top, right, bottom, left), (w, h), _scale) =
                                edit_state.layout.calculate_geometry(self.screen_w, self.screen_h);
                            Task::batch(vec![
                                layer_cmd::set_size(
                                    self.widget_surface_id,
                                    Some(w),
                                    Some(h),
                                ),
                                layer_cmd::set_anchor(self.widget_surface_id, anchor),
                                layer_cmd::set_margin(
                                    self.widget_surface_id,
                                    top,
                                    right,
                                    bottom,
                                    left,
                                ),
                            ])
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
                let old_output_clean = self.config.display.output.as_deref().map(clean_display_name);
                let new_output_clean = new_config.display.output.as_deref().map(clean_display_name);
                let display_changed = old_output_clean != new_output_clean;
                let location_changed =
                    (self.config.weather.latitude - new_config.weather.latitude).abs() > 0.0001
                        || (self.config.weather.longitude - new_config.weather.longitude).abs()
                            > 0.0001;

                self.config = new_config;

                let mut tasks = Vec::new();

                if location_changed {
                    self.weather = None;
                    self.weather_error = false;
                    let s = self.weather_service.clone();
                    let lat = self.config.weather.latitude;
                    let lon = self.config.weather.longitude;
                    tasks.push(Task::future(async move {
                        let res = s.get_weather(lat, lon).await;
                        cosmic::Action::from(Message::WeatherUpdated(res))
                    }));
                }

                if display_changed {
                    info!(
                        "Display target changed to {:?}. Recreating widget surface.",
                        self.config.display.output
                    );
                    tasks.push(self.recreate_widget());
                    return Task::batch(tasks);
                }

                if let WidgetState::Normal = self.state {
                    let (anchor, (top, right, bottom, left), (w, h), _) =
                        self.config.layout.calculate_geometry(self.screen_w, self.screen_h);
                    tasks.push(layer_cmd::set_anchor(
                        self.widget_surface_id,
                        anchor,
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
                        Some(w),
                        Some(h),
                    ));
                    tasks.push(layer_cmd::set_input_zone(
                        self.widget_surface_id,
                        Some(widget::content_bounds(&self.config)),
                    ));
                }

                Task::batch(tasks)
            }

            Message::WaylandOutput(output_event, wl_output) => {
                match output_event {
                    OutputEvent::Created(info_opt) => {
                        let name = info_opt.and_then(|info| info.name);
                        if let Some(entry) = self.outputs.iter_mut().find(|e| e.output == wl_output) {
                            if name.is_some() {
                                entry.name = name;
                            }
                        } else {
                            self.outputs.push(OutputEntry {
                                name,
                                output: wl_output,
                            });
                        }
                    }
                    OutputEvent::InfoUpdate(info) => {
                        if let Some((w, h)) = info.logical_size {
                            if w > 0 && h > 0 {
                                self.screen_w = w as u32;
                                self.screen_h = h as u32;
                            }
                        }
                        if let Some(entry) = self.outputs.iter_mut().find(|e| e.output == wl_output) {
                            entry.name = info.name;
                        } else {
                            self.outputs.push(OutputEntry {
                                name: info.name,
                                output: wl_output,
                            });
                        }
                    }
                    OutputEvent::Removed => {
                        self.outputs.retain(|e| e.output != wl_output);
                    }
                }

                // If user configured a specific display output (e.g. "DP-2"),
                // and that display has just become available, relocate the widget.
                if let Some(target) = &self.config.display.output {
                    let clean_target = clean_display_name(target);
                    if !clean_target.is_empty() {
                        let is_available = self.outputs.iter().any(|e| {
                            e.name
                                .as_deref()
                                .map(|n| clean_display_name(n).eq_ignore_ascii_case(&clean_target))
                                .unwrap_or(false)
                        });
                        let current_clean = self
                            .current_assigned_output
                            .as_deref()
                            .map(clean_display_name);
                        if is_available && current_clean.as_deref() != Some(clean_target.as_str()) {
                            info!(
                                "Configured target display '{}' discovered! Relocating widget.",
                                clean_target
                            );
                            return self.recreate_widget();
                        }
                    }
                }

                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let tick =
            cosmic::iced::Subscription::run(toodle::clock::next_second_tick).map(|_| Message::Tick);
        let watcher = cosmic::iced::Subscription::run(Config::watch).map(Message::ConfigReloaded);
        let weather_sub =
            cosmic::iced::Subscription::run(weather_update_stream).map(|_| Message::FetchWeather);

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

        let wayland_outputs = cosmic::iced::event::listen_with(|event, _status, _id| {
            if let Event::PlatformSpecific(PlatformSpecific::Wayland(WaylandEvent::Output(
                output_event,
                wl_output,
            ))) = event
            {
                Some(Message::WaylandOutput(output_event, wl_output))
            } else {
                None
            }
        });

        Subscription::batch(vec![tick, watcher, weather_sub, escape_key, wayland_outputs])
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
