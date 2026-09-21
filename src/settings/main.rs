use cosmic::app::{Core, Settings, Task};
use cosmic::iced::widget::{column, container, row, scrollable, text};
use cosmic::iced::{Alignment, Border, Color, Length, Shadow, Size};
use cosmic::widget::{button, mouse_area, slider, text_input, toggler};
use cosmic::{Application, Element};

use toodle::config::theme::{COLOR_PALETTE_16, THEME_PRESETS};
use toodle::config::{Anchor, Config, TemperatureUnit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsTab {
    Appearance,
    Layout,
    Weather,
    Display,
}

struct SettingsApp {
    core: Core,
    config: Config,
    active_tab: SettingsTab,
    location_input: String,
    lat_input: String,
    lon_input: String,
    output_input: String,
    status_message: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    SelectTab(SettingsTab),
    SetTheme(String),
    SetColor(String),
    SetFontScale(f32),
    SetTextShadow(bool),
    SetAnchor(Anchor),
    SetMarginX(i32),
    SetMarginY(i32),
    SetWidth(u32),
    SetHeight(u32),
    SetLocationName(String),
    SetLatitude(String),
    SetLongitude(String),
    SetTemperatureUnit(TemperatureUnit),
    SetOutput(String),
    QuickCity(&'static str, f64, f64),
    ApplyLocation,
    Save,
    ResetDefaults,
    Close,
}

impl Application for SettingsApp {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "com.github.wammed.toodle.settings";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let config = Config::load();
        let location_input = config.weather.location_name.clone();
        let lat_input = config.weather.latitude.to_string();
        let lon_input = config.weather.longitude.to_string();
        let output_input = config.display.output.clone().unwrap_or_default();

        let app = Self {
            core,
            config,
            active_tab: SettingsTab::Appearance,
            location_input,
            lat_input,
            lon_input,
            output_input,
            status_message: None,
        };

        (app, Task::none())
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::SelectTab(tab) => {
                self.active_tab = tab;
                self.status_message = None;
                Task::none()
            }

            Message::SetTheme(theme_name) => {
                if let Some(preset) = THEME_PRESETS.iter().find(|p| p.name == theme_name) {
                    self.config.appearance.theme = preset.name.to_string();
                    self.config.appearance.color = preset.default_color.to_string();
                } else {
                    self.config.appearance.theme = theme_name;
                }
                let _ = self.config.save();
                Task::none()
            }

            Message::SetColor(hex) => {
                self.config.appearance.color = hex;
                let _ = self.config.save();
                Task::none()
            }

            Message::SetFontScale(scale) => {
                self.config.appearance.font_scale = scale;
                let _ = self.config.save();
                Task::none()
            }

            Message::SetTextShadow(enabled) => {
                self.config.appearance.text_shadow = enabled;
                let _ = self.config.save();
                Task::none()
            }

            Message::SetAnchor(anchor) => {
                self.config.layout.anchor = anchor;
                let _ = self.config.save();
                Task::none()
            }

            Message::SetMarginX(x) => {
                self.config.layout.margin_x = x.max(0);
                let _ = self.config.save();
                Task::none()
            }

            Message::SetMarginY(y) => {
                self.config.layout.margin_y = y.max(0);
                let _ = self.config.save();
                Task::none()
            }

            Message::SetWidth(w) => {
                self.config.layout.width = w.clamp(150, 2000);
                let _ = self.config.save();
                Task::none()
            }

            Message::SetHeight(h) => {
                self.config.layout.height = h.clamp(60, 1200);
                let _ = self.config.save();
                Task::none()
            }

            Message::SetLocationName(name) => {
                self.location_input = name.clone();
                self.config.weather.location_name = name;
                Task::none()
            }

            Message::SetLatitude(lat_str) => {
                self.lat_input = lat_str.clone();
                if let Ok(val) = lat_str.parse::<f64>() {
                    self.config.weather.latitude = val;
                }
                Task::none()
            }

            Message::SetLongitude(lon_str) => {
                self.lon_input = lon_str.clone();
                if let Ok(val) = lon_str.parse::<f64>() {
                    self.config.weather.longitude = val;
                }
                Task::none()
            }

            Message::SetTemperatureUnit(unit) => {
                self.config.weather.temperature_unit = unit;
                let _ = self.config.save();
                Task::none()
            }

            Message::SetOutput(out) => {
                self.output_input = out.clone();
                if out.trim().is_empty() {
                    self.config.display.output = None;
                } else {
                    self.config.display.output = Some(out.trim().to_string());
                }
                let _ = self.config.save();
                Task::none()
            }

            Message::QuickCity(name, lat, lon) => {
                self.location_input = name.to_string();
                self.lat_input = lat.to_string();
                self.lon_input = lon.to_string();
                self.config.weather.location_name = name.to_string();
                self.config.weather.latitude = lat;
                self.config.weather.longitude = lon;
                let _ = self.config.save();
                self.status_message = Some(format!(
                    "Location updated to {}. Live weather updating!",
                    name
                ));
                Task::none()
            }

            Message::ApplyLocation => {
                if let Ok(val) = self.lat_input.parse::<f64>() {
                    self.config.weather.latitude = val;
                }
                if let Ok(val) = self.lon_input.parse::<f64>() {
                    self.config.weather.longitude = val;
                }
                self.config.weather.location_name = self.location_input.clone();
                let _ = self.config.save();
                self.status_message = Some(format!(
                    "Location applied: {}. Live weather updating!",
                    self.config.weather.location_name
                ));
                Task::none()
            }

            Message::Save => {
                match self.config.save() {
                    Ok(_) => {
                        self.status_message =
                            Some("Configuration saved. Active widget reloaded live!".into());
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Error saving config: {}", e));
                    }
                }
                Task::none()
            }

            Message::ResetDefaults => {
                self.config = Config::default();
                self.location_input = self.config.weather.location_name.clone();
                self.lat_input = self.config.weather.latitude.to_string();
                self.lon_input = self.config.weather.longitude.to_string();
                self.output_input = self.config.display.output.clone().unwrap_or_default();
                let _ = self.config.save();
                self.status_message = Some("Settings reset to defaults and applied live!".into());
                Task::none()
            }

            Message::Close => {
                if let Some(id) = self.core.main_window_id() {
                    cosmic::iced::window::close(id)
                } else {
                    Task::none()
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let app_title = row![
            cosmic::widget::icon::from_name("com.github.wammed.toodle").size(28),
            text("Toodle Settings").size(24),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        // Tab selection bar
        let tab_btn = |tab: SettingsTab, label: &'static str| {
            if self.active_tab == tab {
                button::suggested(label)
                    .padding([8, 16])
                    .on_press(Message::SelectTab(tab))
            } else {
                button::standard(label)
                    .padding([8, 16])
                    .on_press(Message::SelectTab(tab))
            }
        };

        let tab_bar = row![
            tab_btn(SettingsTab::Appearance, "Appearance"),
            tab_btn(SettingsTab::Layout, "Layout"),
            tab_btn(SettingsTab::Weather, "Weather"),
            tab_btn(SettingsTab::Display, "Display"),
        ]
        .spacing(8);

        // Tab Content
        let tab_content: Element<Self::Message> = match self.active_tab {
            SettingsTab::Appearance => self.view_appearance_tab(),
            SettingsTab::Layout => self.view_layout_tab(),
            SettingsTab::Weather => self.view_weather_tab(),
            SettingsTab::Display => self.view_display_tab(),
        };

        let scrollable_content = scrollable(tab_content)
            .width(Length::Fill)
            .height(Length::Fill);

        // Bottom Action Bar
        let status_view: Element<Self::Message> = if let Some(ref status) = self.status_message {
            container(text(status).size(13))
                .style(|_| container::Style {
                    text_color: Some(Color::from_rgb(0.3, 0.85, 0.45)),
                    ..Default::default()
                })
                .into()
        } else {
            text("").size(13).into()
        };

        let bottom_actions = row![
            status_view,
            row![
                button::suggested("Save & Apply")
                    .padding([10, 18])
                    .on_press(Message::Save),
                button::standard("Reset Defaults")
                    .padding([10, 14])
                    .on_press(Message::ResetDefaults),
                button::standard("Close")
                    .padding([10, 14])
                    .on_press(Message::Close),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
        ]
        .spacing(16)
        .align_y(Alignment::Center);

        let main_layout = column![app_title, tab_bar, scrollable_content, bottom_actions,]
            .spacing(16)
            .padding(24);

        container(main_layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Color::from_rgb(0.12, 0.13, 0.17).into()),
                ..Default::default()
            })
            .into()
    }
}

impl SettingsApp {
    fn view_appearance_tab(&self) -> Element<'_, Message> {
        let cur_theme = &self.config.appearance.theme;
        let cur_color = &self.config.appearance.color;

        // Theme presets grid
        let mut theme_buttons = column![].spacing(8);
        for preset in THEME_PRESETS {
            let is_selected = cur_theme.eq_ignore_ascii_case(preset.name);
            let btn = if is_selected {
                button::suggested(format!("✓ {}", preset.name))
                    .width(Length::Fixed(160.0))
                    .padding([8, 12])
                    .on_press(Message::SetTheme(preset.name.to_string()))
            } else {
                button::standard(preset.name)
                    .width(Length::Fixed(160.0))
                    .padding([8, 12])
                    .on_press(Message::SetTheme(preset.name.to_string()))
            };

            let row_item = row![
                btn,
                container(
                    text(format!(
                        "[{}] — {}",
                        preset.font_kind.label(),
                        preset.description
                    ))
                    .size(13)
                )
                .style(|_| container::Style {
                    text_color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
                    ..Default::default()
                })
            ]
            .spacing(14)
            .align_y(Alignment::Center);

            theme_buttons = theme_buttons.push(row_item);
        }

        // 16-Color Palette Grid (8 items per row x 2 rows)
        let mut color_rows = column![].spacing(8);
        let mut row_1 = row![].spacing(8);
        let mut row_2 = row![].spacing(8);

        for (idx, pal) in COLOR_PALETTE_16.iter().enumerate() {
            let is_selected = cur_color.eq_ignore_ascii_case(pal.hex);
            let swatch_color =
                toodle::config::theme::parse_hex_color(pal.hex).unwrap_or(Color::WHITE);

            let swatch = container(text(if is_selected { "✓" } else { "" }).size(14))
                .width(Length::Fixed(36.0))
                .height(Length::Fixed(36.0))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .style(move |_| cosmic::iced::widget::container::Style {
                    background: Some(swatch_color.into()),
                    border: Border {
                        color: if is_selected {
                            Color::WHITE
                        } else {
                            Color::from_rgba(0.4, 0.4, 0.4, 0.4)
                        },
                        width: if is_selected { 2.5 } else { 1.0 },
                        radius: 8.0.into(),
                    },
                    text_color: Some(if pal.hex == "#FFFFFF" || pal.hex == "#E2E8F0" {
                        Color::BLACK
                    } else {
                        Color::WHITE
                    }),
                    shadow: Shadow::default(),
                    ..Default::default()
                });

            let btn = mouse_area(swatch).on_press(Message::SetColor(pal.hex.to_string()));

            if idx < 8 {
                row_1 = row_1.push(btn);
            } else {
                row_2 = row_2.push(btn);
            }
        }
        color_rows = color_rows.push(row_1).push(row_2);

        // Font scale slider
        let scale_percent = (self.config.appearance.font_scale * 100.0).round() as i32;
        let scale_slider = slider(
            0.3..=10.0,
            self.config.appearance.font_scale,
            Message::SetFontScale,
        )
        .step(0.05_f32)
        .width(Length::Fixed(340.0));

        // Text shadow toggler
        let shadow_toggle = toggler(self.config.appearance.text_shadow)
            .label("Enable Text Shadow".to_string())
            .on_toggle(Message::SetTextShadow);

        column![
            text("Theme Presets").size(17),
            theme_buttons,
            text("Color Palette (16 Curated Colors)").size(17),
            color_rows,
            row![
                text(format!("Font Scale: {}%", scale_percent)).size(15),
                scale_slider
            ]
            .spacing(16)
            .align_y(Alignment::Center),
            shadow_toggle,
        ]
        .spacing(18)
        .into()
    }

    fn view_layout_tab(&self) -> Element<'_, Message> {
        let cur_anchor = self.config.layout.anchor;

        let anchor_btn = |a: Anchor, label: &'static str| {
            if cur_anchor == a {
                button::suggested(label)
                    .width(Length::Fixed(120.0))
                    .padding([8, 12])
                    .on_press(Message::SetAnchor(a))
            } else {
                button::standard(label)
                    .width(Length::Fixed(120.0))
                    .padding([8, 12])
                    .on_press(Message::SetAnchor(a))
            }
        };

        let anchor_row = row![
            anchor_btn(Anchor::TopLeft, "TopLeft"),
            anchor_btn(Anchor::TopRight, "TopRight"),
            anchor_btn(Anchor::BottomLeft, "BottomLeft"),
            anchor_btn(Anchor::BottomRight, "BottomRight"),
        ]
        .spacing(10);

        let margin_x_slider = slider(
            0..=2560,
            self.config.layout.margin_x.clamp(0, 2560),
            Message::SetMarginX,
        )
        .step(2)
        .width(Length::Fixed(340.0));

        let margin_y_slider = slider(
            0..=1440,
            self.config.layout.margin_y.clamp(0, 1440),
            Message::SetMarginY,
        )
        .step(2)
        .width(Length::Fixed(340.0));

        let width_slider = slider(
            150..=1200,
            self.config.layout.width.clamp(150, 1200),
            Message::SetWidth,
        )
        .step(5u32)
        .width(Length::Fixed(340.0));

        let height_slider = slider(
            60..=800,
            self.config.layout.height.clamp(60, 800),
            Message::SetHeight,
        )
        .step(5u32)
        .width(Length::Fixed(340.0));

        column![
            text("Desktop Anchor").size(17),
            anchor_row,
            text("Margins").size(17),
            row![
                text(format!("Margin X: {} px", self.config.layout.margin_x)).size(15),
                margin_x_slider
            ]
            .spacing(16)
            .align_y(Alignment::Center),
            row![
                text(format!("Margin Y: {} px", self.config.layout.margin_y)).size(15),
                margin_y_slider
            ]
            .spacing(16)
            .align_y(Alignment::Center),
            text("Widget Size").size(17),
            row![
                text(format!("Width: {} px", self.config.layout.width)).size(15),
                width_slider
            ]
            .spacing(16)
            .align_y(Alignment::Center),
            row![
                text(format!("Height: {} px", self.config.layout.height)).size(15),
                height_slider
            ]
            .spacing(16)
            .align_y(Alignment::Center),
        ]
        .spacing(18)
        .into()
    }

    fn view_weather_tab(&self) -> Element<'_, Message> {
        let city_btn = |name: &'static str, lat: f64, lon: f64| {
            let is_cur = (self.config.weather.latitude - lat).abs() < 0.05
                && (self.config.weather.longitude - lon).abs() < 0.05;
            if is_cur {
                button::suggested(format!("✓ {}", name))
                    .padding([6, 12])
                    .on_press(Message::QuickCity(name, lat, lon))
            } else {
                button::standard(name)
                    .padding([6, 12])
                    .on_press(Message::QuickCity(name, lat, lon))
            }
        };

        let city_row = row![
            city_btn("Tokyo", 35.6895, 139.6917),
            city_btn("Gifu", 35.4233, 136.7606),
            city_btn("London", 51.5074, -0.1278),
            city_btn("New York", 40.7128, -74.0060),
            city_btn("Paris", 48.8566, 2.3522),
        ]
        .spacing(8);

        let cur_unit = self.config.weather.temperature_unit;
        let unit_c_btn = if cur_unit == TemperatureUnit::Celsius {
            button::suggested("Celsius (°C)").padding([8, 16])
        } else {
            button::standard("Celsius (°C)")
                .padding([8, 16])
                .on_press(Message::SetTemperatureUnit(TemperatureUnit::Celsius))
        };

        let unit_f_btn = if cur_unit == TemperatureUnit::Fahrenheit {
            button::suggested("Fahrenheit (°F)").padding([8, 16])
        } else {
            button::standard("Fahrenheit (°F)")
                .padding([8, 16])
                .on_press(Message::SetTemperatureUnit(TemperatureUnit::Fahrenheit))
        };

        column![
            text("Location Details").size(17),
            row![
                text("Location Name:").size(14).width(Length::Fixed(120.0)),
                text_input("City, Country", &self.location_input)
                    .on_input(Message::SetLocationName)
                    .width(Length::Fixed(280.0))
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                text("Latitude:").size(14).width(Length::Fixed(120.0)),
                text_input("e.g. 35.6895", &self.lat_input)
                    .on_input(Message::SetLatitude)
                    .width(Length::Fixed(280.0))
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                text("Longitude:").size(14).width(Length::Fixed(120.0)),
                text_input("e.g. 139.6917", &self.lon_input)
                    .on_input(Message::SetLongitude)
                    .width(Length::Fixed(280.0))
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                button::suggested("Apply Location & Refresh Weather")
                    .padding([8, 18])
                    .on_press(Message::ApplyLocation)
            ],
            text("Quick City Presets").size(15),
            city_row,
            text("Temperature Unit").size(17),
            row![unit_c_btn, unit_f_btn].spacing(10),
        ]
        .spacing(16)
        .into()
    }

    fn view_display_tab(&self) -> Element<'_, Message> {
        let note = container(text(
            "Specify the Wayland output name (e.g., 'DP-1', 'HDMI-A-1') where the clock widget should appear.\nLeave empty to automatically attach to the primary or active output."
        ).size(13))
        .style(|_| container::Style {
            text_color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.65)),
            ..Default::default()
        });

        column![
            text("Target Display Output").size(17),
            note,
            row![
                text("Output Name:").size(14).width(Length::Fixed(120.0)),
                text_input("e.g. DP-1 (or empty)", &self.output_input)
                    .on_input(Message::SetOutput)
                    .width(Length::Fixed(280.0))
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        ]
        .spacing(16)
        .into()
    }
}

fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt::init();
    cosmic::app::run::<SettingsApp>(Settings::default().size(Size::new(720.0, 780.0)), ())
}
