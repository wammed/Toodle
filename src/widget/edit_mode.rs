use crate::config::{Anchor, LayoutConfig};
use cosmic::Element;
use cosmic::iced::widget::{column, container, row, scrollable, text};
use cosmic::iced::{Alignment, Border, Color, Length, Shadow};
use cosmic::widget::{button, slider};

#[derive(Debug, Clone)]
pub enum EditMessage {
    SetAnchor(Anchor),
    SetMarginX(i32),
    SetMarginY(i32),
    SetWidth(u32),
    SetHeight(u32),
    SetFontScale(f32),
    Save,
    Cancel,
}

#[derive(Debug, Clone)]
pub struct EditState {
    pub layout: LayoutConfig,
    pub font_scale: f32,
}

impl EditState {
    pub fn new(current: LayoutConfig, font_scale: f32) -> Self {
        Self {
            layout: current,
            font_scale,
        }
    }

    pub fn update(&mut self, message: EditMessage) {
        match message {
            EditMessage::SetAnchor(anchor) => {
                self.layout.anchor = anchor;
            }
            EditMessage::SetMarginX(val) => {
                self.layout.margin_x = val.max(0);
            }
            EditMessage::SetMarginY(val) => {
                self.layout.margin_y = val.max(0);
            }
            EditMessage::SetWidth(val) => {
                self.layout.width = val.max(160);
            }
            EditMessage::SetHeight(val) => {
                self.layout.height = val.max(80);
            }
            EditMessage::SetFontScale(val) => {
                self.font_scale = val.clamp(0.3, 10.0);
            }
            EditMessage::Save | EditMessage::Cancel => {}
        }
    }

    pub fn view<'a, Message: From<EditMessage> + Clone + 'static>(&self) -> Element<'a, Message> {
        // Top action bar
        let header_actions = row![
            button::suggested("Done (Save)")
                .width(Length::Fill)
                .on_press(Message::from(EditMessage::Save)),
            button::destructive("Cancel")
                .width(Length::Fill)
                .on_press(Message::from(EditMessage::Cancel)),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let title = text("Edit Widget Layout").size(16);

        let hint = text(
            "Changes are previewed on desktop in real-time.\nPress Esc key anytime to cancel.",
        )
        .size(11);

        // Anchor selection buttons
        let anchor_btn = |label: &'static str, a: Anchor| {
            let is_selected = self.layout.anchor == a;
            let b = if is_selected {
                button::suggested(label)
            } else {
                button::standard(label)
            };
            b.on_press(Message::from(EditMessage::SetAnchor(a)))
        };

        let anchor_row = row![
            text("Anchor:").size(13).width(Length::Fixed(100.0)),
            anchor_btn("TL", Anchor::TopLeft),
            anchor_btn("TR", Anchor::TopRight),
            anchor_btn("BL", Anchor::BottomLeft),
            anchor_btn("BR", Anchor::BottomRight),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        // Margin X slider (0 .. 3840)
        let margin_x_slider = slider(0..=3840, self.layout.margin_x, |val| {
            Message::from(EditMessage::SetMarginX(val))
        })
        .width(Length::Fill);

        let margin_x_row = row![
            text(format!("Margin X: {} px", self.layout.margin_x))
                .size(13)
                .width(Length::Fixed(130.0)),
            margin_x_slider,
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        // Margin Y slider (0 .. 2160)
        let margin_y_slider = slider(0..=2160, self.layout.margin_y, |val| {
            Message::from(EditMessage::SetMarginY(val))
        })
        .width(Length::Fill);

        let margin_y_row = row![
            text(format!("Margin Y: {} px", self.layout.margin_y))
                .size(13)
                .width(Length::Fixed(130.0)),
            margin_y_slider,
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        // Font scale slider (30% .. 1000%)
        let scale_percent = (self.font_scale * 100.0).round() as i32;
        let scale_slider = slider(30..=1000, scale_percent, |val| {
            Message::from(EditMessage::SetFontScale(val as f32 / 100.0))
        })
        .width(Length::Fill);

        let scale_row = row![
            text(format!("Font Scale: {}%", scale_percent))
                .size(13)
                .width(Length::Fixed(130.0)),
            scale_slider,
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        // Width slider (160 .. 3840)
        let width_slider = slider(160..=3840, self.layout.width, |val| {
            Message::from(EditMessage::SetWidth(val))
        })
        .width(Length::Fill);

        let width_row = row![
            text(format!("Width: {} px", self.layout.width))
                .size(13)
                .width(Length::Fixed(130.0)),
            width_slider,
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        // Height slider (80 .. 2160)
        let height_slider = slider(80..=2160, self.layout.height, |val| {
            Message::from(EditMessage::SetHeight(val))
        })
        .width(Length::Fill);

        let height_row = row![
            text(format!("Height: {} px", self.layout.height))
                .size(13)
                .width(Length::Fixed(130.0)),
            height_slider,
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let controls = column![
            title,
            header_actions,
            hint,
            anchor_row,
            margin_x_row,
            margin_y_row,
            scale_row,
            width_row,
            height_row,
        ]
        .spacing(10)
        .padding(14);

        let scroll = scrollable(controls)
            .width(Length::Fill)
            .height(Length::Fill);

        container(scroll)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Color::from_rgb(0.12, 0.13, 0.17).into()),
                border: Border {
                    color: Color::from_rgb(0.35, 0.55, 0.90),
                    width: 1.5,
                    radius: 12.0.into(),
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                    offset: cosmic::iced::Vector::new(0.0, 6.0),
                    blur_radius: 16.0,
                },
                text_color: Some(Color::WHITE),
                ..Default::default()
            })
            .into()
    }
}
