use crate::config::{GridPosition, LayoutConfig, get_size_stage};
use cosmic::Element;
use cosmic::iced::widget::{column, container, row, scrollable, text};
use cosmic::iced::{Alignment, Border, Color, Length, Shadow};
use cosmic::widget::button;

#[derive(Debug, Clone)]
pub enum EditMessage {
    SetGridPosition(GridPosition),
    SetSizeStage(u8),
    Save,
    Cancel,
}

#[derive(Debug, Clone)]
pub struct EditState {
    pub layout: LayoutConfig,
    pub font_scale: f32,
}

impl EditState {
    pub fn new(current: LayoutConfig) -> Self {
        let stage_info = get_size_stage(current.size_stage);
        Self {
            layout: current,
            font_scale: stage_info.font_scale,
        }
    }

    pub fn update(&mut self, message: EditMessage) {
        match message {
            EditMessage::SetGridPosition(pos) => {
                self.layout.grid_position = pos;
            }
            EditMessage::SetSizeStage(stage) => {
                let stage = stage.clamp(1, 10);
                self.layout.size_stage = stage;
                let info = get_size_stage(stage);
                self.font_scale = info.font_scale;
            }
            EditMessage::Save | EditMessage::Cancel => {}
        }
    }

    pub fn view<'a, Message: From<EditMessage> + Clone + 'static>(&self) -> Element<'a, Message> {
        // Top action bar
        let header_actions = row![
            button::suggested("Done (Save)")
                .padding([6, 14])
                .on_press(Message::from(EditMessage::Save)),
            button::standard("Cancel")
                .padding([6, 12])
                .on_press(Message::from(EditMessage::Cancel)),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let title = text("Edit Position & Size").size(16);
        let hint = container(text(
            "Select one of 9 display zones (3x3 grid) and a size stage (up to 2560x1440 WQHD). Font scale adapts automatically."
        ).size(12))
        .style(|_| container::Style {
            text_color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.65)),
            ..Default::default()
        });

        // 9-Zone Grid (3 columns × 3 rows)
        let cur_pos = self.layout.grid_position;
        let grid_btn = |pos: GridPosition, label: &'static str| {
            let is_selected = cur_pos == pos;
            let b = if is_selected {
                button::suggested(label)
            } else {
                button::standard(label)
            };
            b.width(Length::Fill)
                .padding([8, 2])
                .on_press(Message::from(EditMessage::SetGridPosition(pos)))
        };

        let pos_header = row![
            text("Screen Position (9-Zone Grid)").size(13),
            text(format!("Active: {}", cur_pos.label())).size(12),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        // Row 0 (Top)
        let row_top = row![
            grid_btn(GridPosition::TopLeft, "Top-L"),
            grid_btn(GridPosition::TopCenter, "Top-C"),
            grid_btn(GridPosition::TopRight, "Top-R"),
        ]
        .spacing(6);

        // Row 1 (Middle)
        let row_mid = row![
            grid_btn(GridPosition::MiddleLeft, "Mid-L"),
            grid_btn(GridPosition::Center, "Center"),
            grid_btn(GridPosition::MiddleRight, "Mid-R"),
        ]
        .spacing(6);

        // Row 2 (Bottom)
        let row_bot = row![
            grid_btn(GridPosition::BottomLeft, "Bot-L"),
            grid_btn(GridPosition::BottomCenter, "Bot-C"),
            grid_btn(GridPosition::BottomRight, "Bot-R"),
        ]
        .spacing(6);

        let grid_section = column![pos_header, row_top, row_mid, row_bot,].spacing(6);

        // 10-Stage Size Selection (1 ..= 10)
        let cur_stage = self.layout.size_stage.clamp(1, 10);
        let active_info = get_size_stage(cur_stage);

        let size_header = row![
            text("Widget Size (10 Stages)").size(13),
            text(format!(
                "Stage {}: {} ({}×{}, {:.2}x)",
                active_info.stage,
                active_info.label,
                active_info.width,
                active_info.height,
                active_info.font_scale
            ))
            .size(12),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        let stage_btn = |stage: u8, label: &'static str| {
            let is_selected = cur_stage == stage;
            let b = if is_selected {
                button::suggested(label)
            } else {
                button::standard(label)
            };
            b.width(Length::Fill)
                .padding([7, 2])
                .on_press(Message::from(EditMessage::SetSizeStage(stage)))
        };

        // Stages 1..=5
        let stages_row_1 = row![
            stage_btn(1, "1: 280"),
            stage_btn(2, "2: 380"),
            stage_btn(3, "3: 490"),
            stage_btn(4, "4: 620"),
            stage_btn(5, "5: 780"),
        ]
        .spacing(6);

        // Stages 6..=10
        let stages_row_2 = row![
            stage_btn(6, "6: 960"),
            stage_btn(7, "7: 1180"),
            stage_btn(8, "8: 1440"),
            stage_btn(9, "9: 1740"),
            stage_btn(10, "10: 2060"),
        ]
        .spacing(6);

        let size_section = column![size_header, stages_row_1, stages_row_2,].spacing(6);

        let controls = column![title, header_actions, hint, grid_section, size_section,]
            .spacing(14)
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
