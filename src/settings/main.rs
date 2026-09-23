use cosmic::app::{Core, Settings, Task};
use cosmic::iced::widget::{column, container, row, scrollable, text};
use cosmic::iced::{Alignment, Border, Color, Length, Shadow, Size};
use cosmic::widget::{button, mouse_area, text_input, toggler};
use cosmic::{Application, Element};

use toodle::config::theme::{COLOR_PALETTE_16, THEME_PRESETS};
use toodle::config::{Config, TemperatureUnit};
use toodle::display::{DetectedDisplay, clean_display_name, detect_displays};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    Appearance,
    Weather,
    Display,
    About,
}

#[derive(Debug, Clone, Default)]
pub struct Flags {
    pub initial_tab: SettingsTab,
    pub showing_licenses: bool,
}

struct SettingsApp {
    core: Core,
    config: Config,
    active_tab: SettingsTab,
    showing_licenses: bool,
    location_input: String,
    lat_input: String,
    lon_input: String,
    output_input: String,
    detected_displays: Vec<DetectedDisplay>,
    status_message: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    SelectTab(SettingsTab),
    ToggleLicenseView(bool),
    OpenUrl(String),
    SetTheme(String),
    SetColor(String),
    SetTextShadow(bool),
    SetLocationName(String),
    SetLatitude(String),
    SetLongitude(String),
    SetTemperatureUnit(TemperatureUnit),
    SetOutput(String),
    RefreshDisplays,
    QuickCity(&'static str, f64, f64),
    ApplyLocation,
    Save,
    ResetDefaults,
    Close,
}

impl Application for SettingsApp {
    type Executor = cosmic::executor::Default;
    type Flags = Flags;
    type Message = Message;
    const APP_ID: &'static str = "com.github.wammed.toodle.settings";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(mut core: Core, flags: Self::Flags) -> (Self, Task<Self::Message>) {
        core.set_auto_blur(Default::default());
        let mut config = Config::load();
        // Sanitize any existing ANSI escapes from config.display.output
        if let Some(ref out) = config.display.output {
            let cleaned = clean_display_name(out);
            if cleaned.is_empty() {
                config.display.output = None;
            } else {
                config.display.output = Some(cleaned);
            }
        }
        let location_input = config.weather.location_name.clone();
        let lat_input = config.weather.latitude.to_string();
        let lon_input = config.weather.longitude.to_string();
        let output_input = config.display.output.clone().unwrap_or_default();
        let detected_displays = detect_displays();

        let app = Self {
            core,
            config,
            active_tab: flags.initial_tab,
            showing_licenses: flags.showing_licenses,
            location_input,
            lat_input,
            lon_input,
            output_input,
            detected_displays,
            status_message: None,
        };

        let font_tasks: Vec<_> = toodle::clock::fonts::embedded_fonts()
            .into_iter()
            .map(|bytes| cosmic::iced::font::load(bytes).discard())
            .collect();

        (app, Task::batch(font_tasks))
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::SelectTab(tab) => {
                self.active_tab = tab;
                self.showing_licenses = false;
                self.status_message = None;
                if tab == SettingsTab::Display {
                    self.detected_displays = detect_displays();
                }
                Task::none()
            }

            Message::ToggleLicenseView(show) => {
                self.showing_licenses = show;
                Task::none()
            }

            Message::OpenUrl(url) => {
                let _ = std::process::Command::new("xdg-open").arg(url).spawn();
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

            Message::SetTextShadow(enabled) => {
                self.config.appearance.text_shadow = enabled;
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
                let cleaned = clean_display_name(&out);
                self.output_input = cleaned.clone();
                let display_desc = if cleaned.is_empty() {
                    self.config.display.output = None;
                    "Default (Active Display)".to_string()
                } else {
                    self.config.display.output = Some(cleaned.clone());
                    cleaned
                };
                let _ = self.config.save();
                self.status_message = Some(format!(
                    "Target display set to '{}'. Widget relocated live!",
                    display_desc
                ));
                Task::none()
            }

            Message::RefreshDisplays => {
                self.detected_displays = detect_displays();
                self.status_message = Some("Refreshed connected displays list.".to_string());
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
            tab_btn(SettingsTab::Weather, "Weather"),
            tab_btn(SettingsTab::Display, "Display"),
            tab_btn(SettingsTab::About, "About"),
        ]
        .spacing(8);

        // Tab Content
        let content_view: Element<Self::Message> =
            if self.active_tab == SettingsTab::About && self.showing_licenses {
                self.view_license_view()
            } else {
                let tab_content: Element<Self::Message> = match self.active_tab {
                    SettingsTab::Appearance => self.view_appearance_tab(),
                    SettingsTab::Weather => self.view_weather_tab(),
                    SettingsTab::Display => self.view_display_tab(),
                    SettingsTab::About => self.view_about_tab(),
                };

                scrollable(tab_content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            };

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

        let bottom_actions = if self.active_tab == SettingsTab::About {
            row![
                status_view,
                row![
                    button::standard("Close")
                        .padding([10, 14])
                        .on_press(Message::Close),
                ]
                .spacing(10)
                .align_y(Alignment::Center)
            ]
            .spacing(16)
            .align_y(Alignment::Center)
        } else {
            row![
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
            .align_y(Alignment::Center)
        };

        let main_layout = column![app_title, tab_bar, content_view, bottom_actions,]
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

        // Text shadow toggler
        let shadow_toggle = toggler(self.config.appearance.text_shadow)
            .label("Enable Text Shadow".to_string())
            .on_toggle(Message::SetTextShadow);

        let layout_tip = container(
            column![
                text("Window Size & Font Scaling (Coupled)").size(15),
                text("Widget size (10 discrete stages from Compact up to 2560x1440 Max WQHD) and font scaling are coupled together without blur or distortion. Right-click the clock widget and select \"Edit Layout\" to change size and grid position.")
                    .size(13),
            ]
            .spacing(4),
        )
        .padding([12, 16])
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Color::from_rgba(0.2, 0.25, 0.35, 0.35).into()),
            border: Border {
                color: Color::from_rgba(0.4, 0.5, 0.7, 0.3),
                width: 1.0,
                radius: 8.0.into(),
            },
            text_color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.75)),
            ..Default::default()
        });

        column![
            text("Theme Presets").size(17),
            theme_buttons,
            text("Color Palette (16 Curated Colors)").size(17),
            color_rows,
            shadow_toggle,
            layout_tip,
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
            "Select the display where the clock widget should appear, or choose Default to automatically attach to the primary/active screen."
        ).size(13))
        .style(|_| container::Style {
            text_color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.65)),
            ..Default::default()
        });

        // Header for detected displays with refresh button
        let detected_header = row![
            text("Connected Displays (Click to select)").size(17),
            button::standard("⟳ Refresh")
                .padding([4, 10])
                .on_press(Message::RefreshDisplays),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        let cur_target = self
            .config
            .display
            .output
            .as_deref()
            .map(clean_display_name)
            .unwrap_or_default();
        let is_default_selected = cur_target.is_empty();

        let mut display_list = column![].spacing(8);

        // Option 1: Default / Active
        let default_btn = if is_default_selected {
            button::suggested("✓ Default (Active / Primary Screen)")
                .width(Length::Fill)
                .padding([10, 16])
                .on_press(Message::SetOutput(String::new()))
        } else {
            button::standard("Default (Active / Primary Screen)")
                .width(Length::Fill)
                .padding([10, 16])
                .on_press(Message::SetOutput(String::new()))
        };
        display_list = display_list.push(default_btn);

        // Detected displays from cosmic-randr
        for disp in &self.detected_displays {
            let is_selected = !cur_target.is_empty() && cur_target.eq_ignore_ascii_case(&disp.name);
            let label = disp.label();
            let primary_tag = if disp.is_primary { " [Primary]" } else { "" };
            let full_label = format!("{}{}", label, primary_tag);

            let btn = if is_selected {
                button::suggested(format!("✓ {}", full_label))
                    .width(Length::Fill)
                    .padding([10, 16])
                    .on_press(Message::SetOutput(disp.name.clone()))
            } else {
                button::standard(full_label)
                    .width(Length::Fill)
                    .padding([10, 16])
                    .on_press(Message::SetOutput(disp.name.clone()))
            };
            display_list = display_list.push(btn);
        }

        if self.detected_displays.is_empty() {
            let empty_hint = container(text(
                "No displays automatically detected. You can manually enter the output name below."
            ).size(13))
            .style(|_| container::Style {
                text_color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.5)),
                ..Default::default()
            });
            display_list = display_list.push(empty_hint);
        }

        // Manual Override section
        let manual_section = column![
            text("Manual Output Name (Optional Override)").size(15),
            row![
                text("Output:").size(14).width(Length::Fixed(80.0)),
                text_input("e.g. DP-1, DP-2, HDMI-A-1", &self.output_input)
                    .on_input(Message::SetOutput)
                    .width(Length::Fixed(260.0)),
                button::standard("Clear (Default)")
                    .padding([8, 12])
                    .on_press(Message::SetOutput(String::new())),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ]
        .spacing(10);

        column![detected_header, note, display_list, manual_section,]
            .spacing(18)
            .into()
    }

    fn view_about_tab(&self) -> Element<'_, Message> {
        // 1. Project Basic Information
        let app_icon = cosmic::widget::icon::from_name("com.github.wammed.toodle").size(48);
        let app_header = row![
            app_icon,
            column![
                row![
                    text("Toodle").size(22),
                    container(text(format!("v{}", env!("CARGO_PKG_VERSION"))).size(12))
                        .padding([2, 8])
                        .style(|_| container::Style {
                            background: Some(Color::from_rgba(0.4, 0.6, 1.0, 0.2).into()),
                            border: Border {
                                color: Color::from_rgba(0.4, 0.6, 1.0, 0.4),
                                width: 1.0,
                                radius: 4.0.into(),
                            },
                            text_color: Some(Color::from_rgb(0.7, 0.85, 1.0)),
                            ..Default::default()
                        }),
                    container(text("MIT License").size(12))
                        .padding([2, 8])
                        .style(|_| container::Style {
                            background: Some(Color::from_rgba(0.2, 0.8, 0.4, 0.15).into()),
                            border: Border {
                                color: Color::from_rgba(0.2, 0.8, 0.4, 0.35),
                                width: 1.0,
                                radius: 4.0.into(),
                            },
                            text_color: Some(Color::from_rgb(0.5, 0.9, 0.6)),
                            ..Default::default()
                        }),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                text("Modern Digital Clock Widget for the System76 COSMIC Desktop Environment")
                    .size(13),
            ]
            .spacing(4)
        ]
        .spacing(16)
        .align_y(Alignment::Center);

        let repo_row = row![
            text("Repository:").size(13).width(Length::Fixed(90.0)),
            button::standard("https://github.com/wammed/Toodle")
                .padding([6, 12])
                .on_press(Message::OpenUrl("https://github.com/wammed/Toodle".into())),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let project_section = container(column![app_header, repo_row].spacing(12))
            .width(Length::Fill)
            .padding([14, 18])
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.14, 0.15, 0.20).into()),
                border: Border {
                    color: Color::from_rgb(0.23, 0.25, 0.34),
                    width: 1.0,
                    radius: 10.0.into(),
                },
                ..Default::default()
            });

        // 2. Bundled Assets & Licenses
        let assets_header = column![
            text("Bundled Fonts & Icons Licenses").size(16),
            container(
                text("The following fonts and vector icons are embedded directly into the binary:")
                    .size(13),
            )
            .style(|_| container::Style {
                text_color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
                ..Default::default()
            }),
        ]
        .spacing(4);

        let asset_cards = column![
            asset_card(
                "Roboto",
                "Display & Clock Font",
                "Christian Robertson, Google LLC",
                "SIL Open Font License 1.1"
            ),
            asset_card(
                "JetBrains Mono NL",
                "Monospace Font",
                "JetBrains s.r.o.",
                "SIL Open Font License 1.1"
            ),
            asset_card(
                "Open Sans",
                "UI Body Font",
                "Steve Matteson",
                "SIL Open Font License 1.1"
            ),
            asset_card(
                "DejaVu Serif",
                "Serif Font",
                "DejaVu fonts team, Bitstream Inc.",
                "Bitstream Vera / DejaVu License"
            ),
            asset_card(
                "Meteocons",
                "Weather Vector Icons",
                "Bas Milius",
                "MIT License"
            ),
        ]
        .spacing(8);

        // 3. Acknowledgements
        let ack_header = text("Acknowledgements").size(16);
        let ack_desc = container(
            text(
                "Built with libcosmic and the Rust open-source ecosystem.\n\
                Special thanks to the upstream libraries and free public API services:"
            )
            .size(13),
        )
        .style(|_| container::Style {
            text_color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.75)),
            ..Default::default()
        });

        let libcosmic_ref = row![
            text("• libcosmic").size(13),
            container(text("MPL-2.0 / MIT").size(11))
                .padding([1, 6])
                .style(|_| container::Style {
                    background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.08).into()),
                    border: Border {
                        color: Color::from_rgba(1.0, 1.0, 1.0, 0.15),
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }),
            button::standard("GitHub ↗")
                .padding([4, 10])
                .on_press(Message::OpenUrl("https://github.com/pop-os/libcosmic".into())),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let openmeteo_ref = row![
            text("• Open-Meteo Weather API").size(13),
            container(text("CC BY 4.0").size(11))
                .padding([1, 6])
                .style(|_| container::Style {
                    background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.08).into()),
                    border: Border {
                        color: Color::from_rgba(1.0, 1.0, 1.0, 0.15),
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }),
            button::standard("Website ↗")
                .padding([4, 10])
                .on_press(Message::OpenUrl("https://open-meteo.com".into())),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let ack_box = container(column![ack_header, ack_desc, libcosmic_ref, openmeteo_ref].spacing(8))
            .padding([14, 18])
            .width(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.14, 0.15, 0.20).into()),
                border: Border {
                    color: Color::from_rgb(0.23, 0.25, 0.34),
                    width: 1.0,
                    radius: 10.0.into(),
                },
                ..Default::default()
            });

        // 4. View Full License Texts Trigger
        let view_licenses_btn = button::suggested("View Full License Texts")
            .padding([12, 24])
            .on_press(Message::ToggleLicenseView(true));

        let view_licenses_row = container(view_licenses_btn)
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .padding([8, 0]);

        column![
            project_section,
            assets_header,
            asset_cards,
            ack_box,
            view_licenses_row,
        ]
        .spacing(18)
        .into()
    }

    fn view_license_view(&self) -> Element<'_, Message> {
        let back_btn = button::standard("← Back to About")
            .padding([8, 16])
            .on_press(Message::ToggleLicenseView(false));

        let header = row![
            back_btn,
            text("Full License Texts").size(18),
        ]
        .spacing(16)
        .align_y(Alignment::Center);

        let license_text_widget = text(FULL_LICENSE_TEXT)
            .font(toodle::clock::fonts::FONT_MONO_REGULAR)
            .size(12);

        let scroll_area = scrollable(
            container(license_text_widget)
                .padding(16)
                .width(Length::Fill),
        )
        .height(Length::Fill)
        .width(Length::Fill);

        let license_box = container(scroll_area)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                // Solid dark background
                background: Some(Color::from_rgb(0.08, 0.09, 0.12).into()),
                border: Border {
                    color: Color::from_rgb(0.24, 0.26, 0.35),
                    width: 1.5,
                    radius: 8.0.into(),
                },
                ..Default::default()
            });

        column![header, license_box]
            .spacing(12)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }
}

fn asset_card<'a>(
    name: &'static str,
    asset_type: &'static str,
    author: &'static str,
    license: &'static str,
) -> Element<'a, Message> {
    let title_line = row![
        text(name).size(15),
        container(text(asset_type).size(11))
            .padding([2, 6])
            .style(|_| container::Style {
                background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.08).into()),
                border: Border {
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.15),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                text_color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
                ..Default::default()
            }),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let details_line = row![
        container(text(author).size(13))
            .width(Length::Fill)
            .style(|_| container::Style {
                text_color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.75)),
                ..Default::default()
            }),
        container(text(license).size(12))
            .padding([2, 8])
            .style(|_| container::Style {
                background: Some(Color::from_rgba(0.2, 0.5, 0.9, 0.2).into()),
                border: Border {
                    color: Color::from_rgba(0.3, 0.6, 1.0, 0.35),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                text_color: Some(Color::from_rgb(0.6, 0.85, 1.0)),
                ..Default::default()
            }),
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    container(column![title_line, details_line].spacing(6))
        .width(Length::Fill)
        .padding([10, 14])
        .style(|_| container::Style {
            background: Some(Color::from_rgb(0.15, 0.16, 0.21).into()),
            border: Border {
                color: Color::from_rgb(0.24, 0.26, 0.35),
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
        .into()
}

const FULL_LICENSE_TEXT: &str = concat!(
    "================================================================================\n",
    "  Toodle - MIT License\n",
    "================================================================================\n\n",
    include_str!("../../LICENSE"),
    "\n\n================================================================================\n",
    "  Roboto, JetBrains Mono NL, Open Sans - SIL Open Font License 1.1\n",
    "================================================================================\n\n",
    include_str!("../../THIRD_PARTY_LICENSES/ROBOTO_OFL.txt"),
    "\n\n================================================================================\n",
    "  DejaVu Serif - Bitstream Vera / DejaVu Fonts License\n",
    "================================================================================\n\n",
    include_str!("../../THIRD_PARTY_LICENSES/DEJAVU_LICENSE.txt"),
    "\n\n================================================================================\n",
    "  Meteocons Weather Icons - MIT License (Bas Milius)\n",
    "================================================================================\n\n",
    include_str!("../../THIRD_PARTY_LICENSES/METEOCONS_LICENSE.txt"),
    "\n\n================================================================================\n\n",
    "For third-party Rust crates and dependencies statically linked at build time,\n",
    "please refer to their respective upstream source repositories or the system\n",
    "package license documentation.\n"
);

pub fn parse_flags_from_args<I, S>(args: I) -> Flags
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut flags = Flags::default();
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        if arg.as_ref() == "--tab" {
            if let Some(val) = iter.next() {
                match val.as_ref().to_ascii_lowercase().as_str() {
                    "about" => {
                        flags.initial_tab = SettingsTab::About;
                        flags.showing_licenses = false;
                    }
                    "license" | "licenses" => {
                        flags.initial_tab = SettingsTab::About;
                        flags.showing_licenses = true;
                    }
                    "appearance" => flags.initial_tab = SettingsTab::Appearance,
                    "weather" => flags.initial_tab = SettingsTab::Weather,
                    "display" => flags.initial_tab = SettingsTab::Display,
                    _ => {}
                }
            }
        }
    }
    flags
}

fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt::init();
    let flags = parse_flags_from_args(std::env::args().skip(1));
    cosmic::app::run::<SettingsApp>(Settings::default().size(Size::new(720.0, 780.0)), flags)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_flags_from_args() {
        let f1 = parse_flags_from_args(vec!["--tab", "about"]);
        assert_eq!(f1.initial_tab, SettingsTab::About);
        assert!(!f1.showing_licenses);

        let f2 = parse_flags_from_args(vec!["--tab", "About"]);
        assert_eq!(f2.initial_tab, SettingsTab::About);
        assert!(!f2.showing_licenses);

        let f3 = parse_flags_from_args(vec!["--tab", "license"]);
        assert_eq!(f3.initial_tab, SettingsTab::About);
        assert!(f3.showing_licenses);

        let f4 = parse_flags_from_args(vec!["--tab", "licenses"]);
        assert_eq!(f4.initial_tab, SettingsTab::About);
        assert!(f4.showing_licenses);

        let f5 = parse_flags_from_args(vec!["--tab", "weather"]);
        assert_eq!(f5.initial_tab, SettingsTab::Weather);

        let f6 = parse_flags_from_args(vec!["--tab", "display"]);
        assert_eq!(f6.initial_tab, SettingsTab::Display);

        let f7 = parse_flags_from_args(vec!["--tab", "appearance"]);
        assert_eq!(f7.initial_tab, SettingsTab::Appearance);

        let f8 = parse_flags_from_args(vec!["--tab", "unknown"]);
        assert_eq!(f8.initial_tab, SettingsTab::Appearance);

        let f9 = parse_flags_from_args(Vec::<String>::new());
        assert_eq!(f9.initial_tab, SettingsTab::Appearance);
    }

    #[test]
    fn test_full_license_text_contains_all_components() {
        assert!(FULL_LICENSE_TEXT.contains("Toodle - MIT License"));
        assert!(FULL_LICENSE_TEXT.contains("Copyright (c) 2026 wammed"));
        assert!(FULL_LICENSE_TEXT.contains("SIL OPEN FONT LICENSE Version 1.1"));
        assert!(FULL_LICENSE_TEXT.contains("Bitstream Vera"));
        assert!(FULL_LICENSE_TEXT.contains("Bas Milius"));
        assert!(FULL_LICENSE_TEXT.contains("statically linked at build time"));
    }
}
