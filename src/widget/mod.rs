pub mod edit_mode;

use crate::config::Config;
use chrono::Local;
use cosmic::iced::widget::{column, container, text};
use cosmic::iced::{Alignment, Border, Color, Length, Rectangle, Shadow};
use cosmic::widget::mouse_area;
use cosmic::Element;
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

pub fn view_widget<'a, Message: From<WidgetMessage> + Clone + 'static>(
    state: &WidgetState,
    config: &Config,
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

    let time_text = text(time_str).size(time_size);
    let date_text = text(date_str).size(date_size);

    let content = column![time_text, date_text]
        .spacing(4)
        .align_x(Alignment::Start);

    if is_editing {
        // In Edit Mode, show a highlighted dashed/solid boundary around the widget on desktop
        container(content)
            .padding(12)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Color::from_rgba(0.2, 0.4, 0.8, 0.15).into()),
                border: Border {
                    color: Color::from_rgb(0.35, 0.65, 1.0),
                    width: 2.0,
                    radius: 8.0.into(),
                },
                shadow: Shadow::default(),
                text_color: Some(Color::WHITE),
                ..Default::default()
            })
            .into()
    } else {
        // In Normal Mode, completely transparent background
        let boxed = container(content)
            .padding(12)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: None,
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
