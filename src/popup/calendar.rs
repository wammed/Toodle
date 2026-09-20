use chrono::{Datelike, Local, NaiveDate};
use cosmic::iced::widget::{column, container, row, text};
use cosmic::iced::{Alignment, Border, Color, Length, Shadow};
use cosmic::widget::button;
use cosmic::Element;
use super::PopupMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalendarState {
    pub year: i32,
    pub month: u32,
}

impl CalendarState {
    pub fn new() -> Self {
        let now = Local::now();
        Self {
            year: now.year(),
            month: now.month(),
        }
    }

    pub fn prev_month(&mut self) {
        if self.month == 1 {
            self.month = 12;
            self.year -= 1;
        } else {
            self.month -= 1;
        }
    }

    pub fn next_month(&mut self) {
        if self.month == 12 {
            self.month = 1;
            self.year += 1;
        } else {
            self.month += 1;
        }
    }

    pub fn jump_today(&mut self) {
        let now = Local::now();
        self.year = now.year();
        self.month = now.month();
    }
}

impl Default for CalendarState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "",
    }
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    let next_month_date = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };

    if let (Some(cur), Some(next)) = (
        NaiveDate::from_ymd_opt(year, month, 1),
        next_month_date,
    ) {
        (next - cur).num_days() as u32
    } else {
        30
    }
}

pub fn view_calendar<'a, Message: From<PopupMessage> + Clone + 'static>(
    state: &CalendarState,
) -> Element<'a, Message> {
    let now = Local::now().date_naive();
    let today_year = now.year();
    let today_month = now.month();
    let today_day = now.day();

    // Navigation and title header
    let title_str = format!("{} {}", month_name(state.month), state.year);
    let title_text = text(title_str).size(26);

    let header_nav = row![
        button::standard("<")
            .padding([8, 16])
            .on_press(Message::from(PopupMessage::CalendarPrevMonth)),
        button::standard("Today")
            .padding([8, 16])
            .on_press(Message::from(PopupMessage::CalendarToday)),
        button::standard(">")
            .padding([8, 16])
            .on_press(Message::from(PopupMessage::CalendarNextMonth)),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let top_bar = row![
        title_text,
        row![
            header_nav,
            button::standard("Close")
                .padding([8, 16])
                .on_press(Message::from(PopupMessage::Close))
        ]
        .spacing(12)
        .align_y(Alignment::Center)
    ]
    .spacing(20)
    .align_y(Alignment::Center);

    // Weekday headers: SUN, MON, TUE, WED, THU, FRI, SAT
    let weekdays = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];
    let mut weekday_row = row![].spacing(8);
    for wd in weekdays {
        let is_weekend = wd == "SUN" || wd == "SAT";
        let color = if is_weekend {
            Color::from_rgb(0.95, 0.45, 0.45)
        } else {
            Color::from_rgba(1.0, 1.0, 1.0, 0.7)
        };
        let label = container(text(wd).size(13))
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .style(move |_| container::Style {
                text_color: Some(color),
                ..Default::default()
            });

        weekday_row = weekday_row.push(label);
    }

    // Days grid calculation
    let first_day = NaiveDate::from_ymd_opt(state.year, state.month, 1).unwrap_or(now);
    let start_offset = first_day.weekday().num_days_from_sunday() as usize; // 0..=6
    let days_count = days_in_month(state.year, state.month) as usize;

    let prev_month = if state.month == 1 { 12 } else { state.month - 1 };
    let prev_year = if state.month == 1 { state.year - 1 } else { state.year };
    let prev_days_count = days_in_month(prev_year, prev_month) as usize;

    let mut grid_rows = column![].spacing(8);

    // 6 rows x 7 days = 42 slots
    for row_idx in 0..6 {
        let mut week_row = row![].spacing(8);
        for col_idx in 0..7 {
            let slot = row_idx * 7 + col_idx;
            let (day_num, is_cur_month, is_today) = if slot < start_offset {
                let num = prev_days_count - (start_offset - slot - 1);
                (num, false, false)
            } else if slot < start_offset + days_count {
                let num = slot - start_offset + 1;
                let is_today = state.year == today_year
                    && state.month == today_month
                    && num as u32 == today_day;
                (num, true, is_today)
            } else {
                let num = slot - (start_offset + days_count) + 1;
                (num, false, false)
            };

            let cell_content = text(format!("{}", day_num)).size(16);

            let cell = if is_today {
                container(cell_content)
                    .width(Length::Fill)
                    .height(Length::Fixed(60.0))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .style(|_theme| container::Style {
                        background: Some(Color::from_rgb(0.24, 0.54, 0.96).into()),
                        border: Border {
                            color: Color::WHITE,
                            width: 1.5,
                            radius: 10.0.into(),
                        },
                        text_color: Some(Color::WHITE),
                        ..Default::default()
                    })
            } else if is_cur_month {
                let is_weekend = col_idx == 0 || col_idx == 6;
                let text_color = if is_weekend {
                    Color::from_rgb(0.95, 0.65, 0.65)
                } else {
                    Color::WHITE
                };

                container(cell_content)
                    .width(Length::Fill)
                    .height(Length::Fixed(60.0))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .style(move |_theme| container::Style {
                        background: Some(Color::from_rgba(0.2, 0.22, 0.28, 0.5).into()),
                        border: Border {
                            color: Color::from_rgba(0.3, 0.35, 0.45, 0.3),
                            width: 1.0,
                            radius: 8.0.into(),
                        },
                        text_color: Some(text_color),
                        ..Default::default()
                    })
            } else {
                container(cell_content)
                    .width(Length::Fill)
                    .height(Length::Fixed(60.0))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .style(|_theme| container::Style {
                        background: None,
                        border: Border::default(),
                        text_color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.25)),
                        ..Default::default()
                    })
            };

            week_row = week_row.push(cell);
        }
        grid_rows = grid_rows.push(week_row);
    }

    let calendar_box = column![top_bar, weekday_row, grid_rows]
        .spacing(18)
        .padding(24);

    container(calendar_box)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calendar_navigation() {
        let mut state = CalendarState { year: 2026, month: 1 };
        state.prev_month();
        assert_eq!(state.year, 2025);
        assert_eq!(state.month, 12);

        state.next_month();
        assert_eq!(state.year, 2026);
        assert_eq!(state.month, 1);
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(2026, 1), 31); // Jan
        assert_eq!(days_in_month(2026, 2), 28); // Feb normal
        assert_eq!(days_in_month(2024, 2), 29); // Feb leap
        assert_eq!(days_in_month(2026, 4), 30); // Apr
    }
}
