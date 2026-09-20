pub mod calendar;
pub mod forecast;

pub use calendar::{view_calendar, CalendarState};
pub use forecast::view_forecast;

use cosmic::iced::widget::{column, container, text};
use cosmic::iced::{Alignment, Border, Color, Length, Shadow};
use cosmic::widget::button;
use cosmic::Element;

#[derive(Debug, Clone)]
pub enum PopupMessage {
    EditLayout,
    OpenSettings,
    OpenCalendar,
    OpenForecast,
    CalendarPrevMonth,
    CalendarNextMonth,
    CalendarToday,
    Close,
    Quit,
}

pub fn view_context_menu<'a, Message: From<PopupMessage> + Clone + 'static>() -> Element<'a, Message> {
    let title = container(text("Toodle Menu").size(20))
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .padding([4, 0])
        .style(|_| container::Style {
            text_color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.8)),
            ..Default::default()
        });

    let items = column![
        title,
        button::standard("Calendar")
            .font_size(18)
            .padding([12, 20])
            .width(Length::Fill)
            .on_press(Message::from(PopupMessage::OpenCalendar)),
        button::standard("Weekly Forecast")
            .font_size(18)
            .padding([12, 20])
            .width(Length::Fill)
            .on_press(Message::from(PopupMessage::OpenForecast)),
        button::standard("Edit Layout")
            .font_size(18)
            .padding([12, 20])
            .width(Length::Fill)
            .on_press(Message::from(PopupMessage::EditLayout)),
        button::standard("Settings")
            .font_size(18)
            .padding([12, 20])
            .width(Length::Fill)
            .on_press(Message::from(PopupMessage::OpenSettings)),
        button::destructive("Quit")
            .font_size(18)
            .padding([12, 20])
            .width(Length::Fill)
            .on_press(Message::from(PopupMessage::Quit)),
    ]
    .spacing(12)
    .width(Length::Fixed(280.0))
    .align_x(Alignment::Center);

    container(items)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .style(|_theme| container::Style {
            // Solid non-transparent dark background
            background: Some(Color::from_rgb(0.13, 0.14, 0.18).into()),
            border: Border {
                color: Color::from_rgb(0.28, 0.31, 0.42),
                width: 1.5,
                radius: 12.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.55),
                offset: cosmic::iced::Vector::new(0.0, 6.0),
                blur_radius: 18.0,
            },
            ..Default::default()
        })
        .into()
}
