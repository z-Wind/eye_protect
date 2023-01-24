use iced::widget::canvas::{Cache, Cursor, Geometry};
use iced::widget::{canvas, container};
use iced::{alignment, application, executor, theme, window};
use iced::{
    Application, Color, Command, Element, Length, Rectangle, Settings, Subscription, Theme, Vector,
};

pub fn main() -> iced::Result {
    EyeProtect::run(Settings {
        window: window::Settings {
            always_on_top: false,
            position: window::Position::Centered,
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

struct EyeProtect {
    value: i32,
    text: Cache,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
}

impl Application for EyeProtect {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: ()) -> (EyeProtect, Command<Self::Message>) {
        let value = 20;
        (
            Self {
                value,
                text: Default::default(),
            },
            iced::window::set_mode(iced::window::Mode::Fullscreen),
        )
    }

    fn title(&self) -> String {
        String::from("Eye Protect")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Tick => {
                self.value -= 1;
                self.text.clear();
            }
        }
        if self.value < 0 {
            return window::close();
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(1000)).map(|_| Message::Tick)
    }

    fn view(&self) -> Element<Self::Message> {
        let canvas = canvas(self as &Self)
            .width(Length::Fill)
            .height(Length::Fill);

        container(canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn style(&self) -> theme::Application {
        fn dark_background(_theme: &Theme) -> application::Appearance {
            application::Appearance {
                background_color: Color::BLACK,
                text_color: Color::WHITE,
            }
        }

        theme::Application::from(dark_background as fn(&Theme) -> _)
    }
}

impl<Message> canvas::Program<Message> for EyeProtect {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let text = self.text.draw(bounds.size(), |frame| {
            let center = frame.center();
            frame.translate(Vector::new(center.x, center.y));

            frame.fill_text(canvas::Text {
                content: format!("{:02} s", self.value),
                color: Color::WHITE,
                size: 150.0,
                horizontal_alignment: alignment::Horizontal::Center,
                vertical_alignment: alignment::Vertical::Center,
                ..canvas::Text::default()
            });
        });

        vec![text]
    }
}
