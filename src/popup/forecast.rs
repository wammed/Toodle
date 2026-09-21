use super::PopupMessage;
use crate::config::Config;
use crate::weather::WeatherData;
use crate::widget::format_temperature;
use chrono::{Local, NaiveDate};
use cosmic::Element;
use cosmic::iced::widget::{column, container, row, text};
use cosmic::iced::{Alignment, Border, Color, Length, Shadow};
use cosmic::widget::button;

pub fn view_forecast<'a, Message: From<PopupMessage> + Clone + 'static>(
    config: &Config,
    weather_data: Option<&WeatherData>,
) -> Element<'a, Message> {
    let today = Local::now().date_naive();

    let title_main = text("7-Day Weather Forecast").size(24);
    let title_sub = container(text(format!("Location: {}", config.weather.location_name)).size(13))
        .style(|_| container::Style {
            text_color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.65)),
            ..Default::default()
        });

    let title_text = column![title_main, title_sub].spacing(4);

    let top_bar = row![
        title_text,
        button::standard("Close")
            .padding([8, 18])
            .on_press(Message::from(PopupMessage::Close))
    ]
    .spacing(20)
    .align_y(Alignment::Center);

    let forecast_list: Element<'a, Message> = if let Some(weather) = weather_data {
        if weather.daily.is_empty() {
            container(text("No daily forecast available.").size(16))
                .width(Length::Fill)
                .height(Length::Fixed(400.0))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into()
        } else {
            let mut list_col = column![].spacing(8);

            for day in &weather.daily {
                let parsed_date = NaiveDate::parse_from_str(&day.date, "%Y-%m-%d").ok();
                let is_today = parsed_date.map(|d| d == today).unwrap_or(false);

                let date_display = if let Some(d) = parsed_date {
                    if is_today {
                        format!("{} (Today)", d.format("%a, %b %d"))
                    } else {
                        d.format("%a, %b %d").to_string()
                    }
                } else {
                    day.date.clone()
                };

                let min_str =
                    format_temperature(day.temp_min_celsius, config.weather.temperature_unit);
                let max_str =
                    format_temperature(day.temp_max_celsius, config.weather.temperature_unit);
                let temp_range = format!("{}  ~  {}", min_str, max_str);

                let date_color = if is_today {
                    Color::from_rgb(0.4, 0.75, 1.0)
                } else {
                    Color::WHITE
                };

                let date_box = container(text(date_display).size(15))
                    .width(Length::FillPortion(3))
                    .style(move |_| container::Style {
                        text_color: Some(date_color),
                        ..Default::default()
                    });

                let icon = crate::weather::wmo_code_to_icon(day.weather_code);
                let icon_text = text(icon.glyph().to_string())
                    .font(crate::clock::fonts::FONT_WEATHER_ICONS)
                    .size(16);
                let moon_glyph = crate::weather::MoonPhase::from_ymd_str(&day.date)
                    .map(|m| m.glyph().to_string())
                    .unwrap_or_default();
                let moon_text = text(moon_glyph)
                    .font(crate::clock::fonts::FONT_WEATHER_ICONS)
                    .size(16);
                let condition_text = text(day.condition_text.clone()).size(14);
                let condition_row = row![icon_text, moon_text, condition_text]
                    .spacing(8)
                    .align_y(Alignment::Center);
                let condition_box = container(condition_row)
                    .width(Length::FillPortion(4))
                    .style(|_| container::Style {
                        text_color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.85)),
                        ..Default::default()
                    });

                let temp_box = container(text(temp_range).size(15))
                    .width(Length::FillPortion(3))
                    .align_x(Alignment::End)
                    .style(|_| container::Style {
                        text_color: Some(Color::from_rgb(1.0, 0.85, 0.5)),
                        ..Default::default()
                    });

                let row_content = row![date_box, condition_box, temp_box]
                    .spacing(12)
                    .align_y(Alignment::Center);

                let card = container(row_content)
                    .padding([12, 16])
                    .width(Length::Fill)
                    .style(move |_theme| container::Style {
                        background: Some(if is_today {
                            Color::from_rgba(0.2, 0.35, 0.6, 0.35).into()
                        } else {
                            Color::from_rgba(0.18, 0.20, 0.26, 0.45).into()
                        }),
                        border: Border {
                            color: if is_today {
                                Color::from_rgba(0.35, 0.6, 1.0, 0.6)
                            } else {
                                Color::from_rgba(0.28, 0.32, 0.42, 0.3)
                            },
                            width: 1.0,
                            radius: 8.0.into(),
                        },
                        ..Default::default()
                    });

                list_col = list_col.push(card);
            }

            list_col.into()
        }
    } else {
        let hint_box =
            container(text("Please check network connectivity or try again later.").size(13))
                .style(|_| container::Style {
                    text_color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.5)),
                    ..Default::default()
                });

        container(
            column![
                text("Weather forecast is currently unavailable.").size(16),
                hint_box,
            ]
            .spacing(8)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fixed(400.0))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .into()
    };

    let main_content = column![top_bar, forecast_list].spacing(20).padding(24);

    container(main_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Color::from_rgb(0.12, 0.13, 0.17).into()),
            border: Border {
                color: Color::from_rgb(0.28, 0.31, 0.42),
                width: 1.5,
                radius: 12.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.6),
                offset: cosmic::iced::Vector::new(0.0, 8.0),
                blur_radius: 24.0,
            },
            ..Default::default()
        })
        .into()
}
