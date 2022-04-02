use iced::{executor, Application, Clipboard, Command, Element, Settings, Subscription, Text};

pub fn main() -> iced::Result {
    let mut settings = Settings::default();
    settings.window.always_on_top = true;
    EyeProtect::run(settings)
}

struct EyeProtect {
    value: i32,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
}

impl Application for EyeProtect {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (EyeProtect, Command<Self::Message>) {
        (EyeProtect { value: 20 }, Command::none())
    }

    fn title(&self) -> String {
        String::from("Eye Protect")
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
        match message {
            Message::Tick => {
                self.value -= 1;
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(1000)).map(|_| Message::Tick)
    }

    fn view(&mut self) -> Element<Self::Message> {
        let mut text = Text::new(format!("{:02} s", self.value));
        text = text
            .color(iced::Color::WHITE)
            .size(150)
            .height(iced::Length::Fill)
            .width(iced::Length::Fill)
            .horizontal_alignment(iced::HorizontalAlignment::Center)
            .vertical_alignment(iced::VerticalAlignment::Center);
        text.into()
    }

    fn mode(&self) -> iced::window::Mode {
        iced::window::Mode::Fullscreen
        // iced::window::Mode::Windowed
    }

    fn background_color(&self) -> iced::Color {
        iced::Color::BLACK
    }

    fn should_exit(&self) -> bool {
        self.value == 0
    }
}
