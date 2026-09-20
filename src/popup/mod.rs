use cosmic::iced::widget::{column, container};
use cosmic::iced::{Border, Color, Length, Shadow};
use cosmic::widget::button;
use cosmic::Element;

#[derive(Debug, Clone)]
pub enum PopupMessage {
    EditLayout,
    OpenSettings,
    OpenCalendar,
    OpenForecast,
    #[allow(dead_code)]
    Close,
    Quit,
}

pub fn view_context_menu<'a, Message: From<PopupMessage> + Clone + 'static>() -> Element<'a, Message> {
    let items = column![
        button::standard("Calendar")
            .width(Length::Fill)
            .on_press(Message::from(PopupMessage::OpenCalendar)),
        button::standard("Weekly Forecast")
            .width(Length::Fill)
            .on_press(Message::from(PopupMessage::OpenForecast)),
        button::standard("Edit Layout")
            .width(Length::Fill)
            .on_press(Message::from(PopupMessage::EditLayout)),
        button::standard("Settings")
            .width(Length::Fill)
            .on_press(Message::from(PopupMessage::OpenSettings)),
        button::destructive("Quit")
            .width(Length::Fill)
            .on_press(Message::from(PopupMessage::Quit)),
    ]
    .spacing(4)
    .padding(6);

    container(items)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            // Solid non-transparent background
            background: Some(Color::from_rgb(0.14, 0.15, 0.19).into()),
            border: Border {
                color: Color::from_rgb(0.28, 0.31, 0.40),
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.45),
                offset: cosmic::iced::Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
            ..Default::default()
        })
        .into()
}
