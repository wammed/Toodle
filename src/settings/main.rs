use cosmic::app::{Core, Task};
use cosmic::iced::Length;
use cosmic::widget::{button, column, container, text};
use cosmic::{Application, Element};

use toodle::config::Config;

struct SettingsApp {
    core: Core,
    config: Config,
}

#[derive(Debug, Clone)]
enum Message {
    SaveAndClose,
}

impl Application for SettingsApp {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "com.github.wammed.toodle.settings";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let config = Config::load();
        (Self { core, config }, Task::none())
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::SaveAndClose => {
                let _ = self.config.save();
                if let Some(id) = self.core.main_window_id() {
                    cosmic::iced::window::close(id)
                } else {
                    Task::none()
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let content = column![
            text::title3("Toodle Settings"),
            text(format!("Theme: {}", self.config.appearance.theme)),
            text(format!("Anchor: {:?}", self.config.layout.anchor)),
            text(format!(
                "Margin: ({}, {})",
                self.config.layout.margin_x, self.config.layout.margin_y
            )),
            text(format!("Location: {}", self.config.weather.location_name)),
            button::standard("Save & Close").on_press(Message::SaveAndClose),
        ]
        .spacing(12);

        container(content)
            .padding(24)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt::init();
    cosmic::app::run::<SettingsApp>(
        cosmic::app::Settings::default()
            .size(cosmic::iced::Size::new(480.0, 400.0)),
        (),
    )
}
