pub mod edit_mode;

use crate::config::{Config, TemperatureUnit};
use crate::weather::WeatherData;
use chrono::Local;
use cosmic::Element;
use cosmic::iced::widget::{column, container, row, text};
use cosmic::iced::{Alignment, Border, Color, Length, Rectangle, Shadow};
use cosmic::widget::mouse_area;
use edit_mode::EditState;

#[derive(Debug, Clone)]
pub enum WidgetState {
    Normal,
    Edit(EditState),
}

#[derive(Debug, Clone)]
pub enum WidgetMessage {
    RightClicked,
}

pub fn format_temperature(temp_celsius: f32, unit: TemperatureUnit) -> String {
    match unit {
        TemperatureUnit::Celsius => format!("{:.0}°C", temp_celsius),
        TemperatureUnit::Fahrenheit => {
            let f = (temp_celsius * 9.0 / 5.0) + 32.0;
            format!("{:.0}°F", f)
        }
    }
}

pub fn view_widget<'a, Message: From<WidgetMessage> + Clone + 'static>(
    state: &WidgetState,
    config: &Config,
    weather: Option<&WeatherData>,
    weather_error: bool,
) -> Element<'a, Message> {
    let now = Local::now();
    let time_str = now.format("%H:%M:%S").to_string();
    let date_str = now.format("%A, %B %d, %Y").to_string();

    let (font_scale, is_editing) = match state {
        WidgetState::Normal => (config.appearance.font_scale, false),
        WidgetState::Edit(edit_state) => (edit_state.font_scale, true),
    };

    let font_scale = font_scale.clamp(0.3, 10.0);
    let time_size = (36.0 * font_scale).round() as u16;
    let date_size = (14.0 * font_scale).round() as u16;

    let (time_font, date_font) =
        crate::config::theme::get_font_pair_for_theme(&config.appearance.theme);

    let time_text = text(time_str).font(time_font).size(time_size);

    let date_text = text(date_str).font(date_font).size(date_size);

    let weather_element: Element<'a, Message> = if let Some(w) = weather {
        let temp_str = format_temperature(
            w.current.temperature_celsius,
            config.weather.temperature_unit,
        );
        let icon = crate::weather::wmo_code_to_icon(w.current.weather_code);
        let icon_str = icon.glyph().to_string();

        let icon_text = text(icon_str)
            .font(crate::clock::fonts::FONT_WEATHER_ICONS)
            .size(date_size);

        let moon = crate::weather::MoonPhase::from_datetime(&now);
        let moon_text = text(moon.glyph().to_string())
            .font(crate::clock::fonts::FONT_WEATHER_ICONS)
            .size(date_size);

        let condition_text = text(w.current.condition_text.clone())
            .font(date_font)
            .size(date_size);

        let temp_text = text(temp_str).font(date_font).size(date_size);

        let icon_spacing = (6.0 * font_scale).round().max(4.0) as u16;
        let item_spacing = (16.0 * font_scale).round().max(12.0) as u16;

        let icons_row = row![icon_text, moon_text]
            .spacing(icon_spacing)
            .align_y(Alignment::Center);

        row![icons_row, condition_text, temp_text]
            .spacing(item_spacing)
            .align_y(Alignment::Center)
            .into()
    } else if weather_error {
        text("Weather unavailable")
            .font(date_font)
            .size(date_size)
            .into()
    } else {
        text("...").font(date_font).size(date_size).into()
    };

    let widget_text_color =
        crate::config::parse_hex_color(&config.appearance.color).unwrap_or(Color::WHITE);

    let text_shadow = if config.appearance.text_shadow {
        Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.65),
            offset: cosmic::iced::Vector::new(1.0, 2.0),
            blur_radius: 6.0,
        }
    } else {
        Shadow::default()
    };

    let content = column![time_text, date_text, weather_element]
        .spacing(6)
        .align_x(Alignment::Start);

    if is_editing {
        // In Edit Mode, show a highlighted dashed/solid boundary around the widget on desktop
        container(content)
            .padding(12)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_theme| container::Style {
                background: Some(Color::from_rgba(0.2, 0.4, 0.8, 0.15).into()),
                border: Border {
                    color: Color::from_rgb(0.35, 0.65, 1.0),
                    width: 2.0,
                    radius: 8.0.into(),
                },
                shadow: text_shadow,
                text_color: Some(widget_text_color),
                ..Default::default()
            })
            .into()
    } else {
        // In Normal Mode, completely transparent background
        let boxed = container(content)
            .padding(12)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_theme| container::Style {
                background: None,
                text_color: Some(widget_text_color),
                shadow: text_shadow,
                ..Default::default()
            });

        // Wrap in mouse area to catch right-click
        mouse_area(boxed)
            .on_right_press(Message::from(WidgetMessage::RightClicked))
            .into()
    }
}

/// Returns the content bounds rectangle for Normal Mode input region
pub fn content_bounds(config: &Config) -> Vec<Rectangle> {
    vec![Rectangle {
        x: 0.0,
        y: 0.0,
        width: config.layout.width as f32,
        height: config.layout.height as f32,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_formatting() {
        assert_eq!(format_temperature(22.0, TemperatureUnit::Celsius), "22°C");
        assert_eq!(format_temperature(0.0, TemperatureUnit::Celsius), "0°C");
        assert_eq!(format_temperature(-5.0, TemperatureUnit::Celsius), "-5°C");

        // 20°C -> 68°F
        assert_eq!(
            format_temperature(20.0, TemperatureUnit::Fahrenheit),
            "68°F"
        );
        // 0°C -> 32°F
        assert_eq!(format_temperature(0.0, TemperatureUnit::Fahrenheit), "32°F");
        // 100°C -> 212°F
        assert_eq!(
            format_temperature(100.0, TemperatureUnit::Fahrenheit),
            "212°F"
        );
    }
}
