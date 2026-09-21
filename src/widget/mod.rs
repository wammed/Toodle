pub mod edit_mode;

use crate::config::{Config, TemperatureUnit};
use crate::weather::WeatherData;
use chrono::Local;
use cosmic::Element;
use cosmic::iced::widget::{column, container, row, svg, text};
use cosmic::iced::{Alignment, Border, Color, Length, Rectangle};
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

    let (stage_info, is_editing, grid_pos) = match state {
        WidgetState::Normal => (
            crate::config::get_size_stage(config.layout.size_stage),
            false,
            config.layout.grid_position,
        ),
        WidgetState::Edit(edit_state) => (
            crate::config::get_size_stage(edit_state.layout.size_stage),
            true,
            edit_state.layout.grid_position,
        ),
    };

    use crate::config::GridPosition;
    let align_x = match grid_pos {
        GridPosition::TopLeft | GridPosition::MiddleLeft | GridPosition::BottomLeft => {
            Alignment::Start
        }
        GridPosition::TopCenter | GridPosition::Center | GridPosition::BottomCenter => {
            Alignment::Center
        }
        GridPosition::TopRight | GridPosition::MiddleRight | GridPosition::BottomRight => {
            Alignment::End
        }
    };
    let align_y = Alignment::Start;

    let time_size = stage_info.time_size;
    let date_size = stage_info.date_size;
    let spacing = stage_info.spacing;
    let padding = stage_info.padding;

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
        let icon_size = (date_size as f32 * 1.2).round();
        let icon_svg = svg(svg::Handle::from_memory(icon.svg_bytes()))
            .width(icon_size)
            .height(icon_size);

        let condition_text = text(w.current.condition_text.clone())
            .font(date_font)
            .size(date_size);

        let temp_text = text(temp_str).font(date_font).size(date_size);

        let item_spacing = (spacing * 2).max(12);

        row![icon_svg, condition_text, temp_text]
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

    let content = column![time_text, date_text, weather_element]
        .spacing(spacing)
        .align_x(align_x);

    if is_editing {
        // In Edit Mode, show a highlighted boundary around the widget on desktop without background fill/blur
        container(content)
            .padding(padding)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(align_x)
            .align_y(align_y)
            .style(move |_theme| container::Style {
                background: None,
                border: Border {
                    color: Color::from_rgb(0.35, 0.65, 1.0),
                    width: 2.0,
                    radius: 8.0.into(),
                },
                text_color: Some(widget_text_color),
                ..Default::default()
            })
            .into()
    } else {
        // In Normal Mode, completely transparent background with no shadow
        let boxed = container(content)
            .padding(padding)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(align_x)
            .align_y(align_y)
            .style(move |_theme| container::Style {
                background: None,
                text_color: Some(widget_text_color),
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
    let info = crate::config::get_size_stage(config.layout.size_stage);
    vec![Rectangle {
        x: 0.0,
        y: 0.0,
        width: info.width as f32,
        height: info.height as f32,
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
