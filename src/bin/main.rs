use clap::Parser;
use iced::widget::{
    canvas,
    canvas::{Cache, Geometry},
    container,
};
use iced::{
    Color, Element, Length, Pixels, Rectangle, Renderer, Subscription, Task, Theme, alignment,
    keyboard, window,
};
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "護眼提醒視窗 (GUI 顯示端)",
    long_about = "這是由守護進程啟動的全螢幕 GUI 倒數視窗，負責執行實際的休息提醒。"
)]
struct Args {
    /// 啟用視窗置頂
    #[arg(short, long, help = "將護眼視窗固定在最上層，防止被其他視窗遮擋")]
    top_enable: bool,

    /// 倒數計時秒數
    #[arg(
        short,
        long,
        default_value_t = 20,
        help = "設定顯示視窗的倒數秒數，歸零後自動關閉視窗"
    )]
    wait_seconds: i32,

    /// 提醒文字內容
    #[arg(short, long, help = "在畫面中央顯示自定義的提醒訊息（支援 ASCII）")]
    remind: Option<String>,
}
pub fn main() -> iced::Result {
    let args = Args::parse();

    iced::application(
        move || {
            (
                EyeProtect {
                    value: args.wait_seconds,
                    text: Cache::default(),
                    remind: args.remind.clone(),
                },
                window::latest().and_then(|id| window::set_mode(id, window::Mode::Fullscreen)),
            )
        },
        EyeProtect::update,
        EyeProtect::view,
    )
    .window(window::Settings {
        level: if args.top_enable {
            window::Level::AlwaysOnTop
        } else {
            window::Level::Normal
        },
        ..Default::default()
    })
    .title("Eye Protect")
    .subscription(EyeProtect::subscription)
    // 修正：明確指定閉包參數類型以解決 "not general enough" 錯誤
    .theme(|_state: &EyeProtect| Theme::Dark)
    .run()
}

struct EyeProtect {
    value: i32,
    text: Cache,
    remind: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    EventOccurred(keyboard::Key),
}

impl EyeProtect {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.value -= 1;
                self.text.clear();

                if self.value < 0 {
                    // 使用官方建議的視窗關閉方式
                    return window::latest().and_then(window::close);
                }
            }
            // 監聽 ESC 鍵退出
            Message::EventOccurred(keyboard::Key::Named(keyboard::key::Named::Escape)) => {
                return window::latest().and_then(window::close);
            }
            _ => {}
        }

        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            // 每秒觸發一次 Tick
            iced::time::every(std::time::Duration::from_millis(1000)).map(|_| Message::Tick),
            // 修正：使用 event::listen 監聽所有事件並過濾出按鍵按下的事件
            iced::event::listen_with(|event, _status, _window_id| match event {
                iced::Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                    Some(Message::EventOccurred(key))
                }
                _ => None,
            }),
        ])
    }

    fn view(&self) -> Element<'_, Message> {
        // 使用 canvas 封裝，並確保填滿
        let canvas_widget = canvas(self).width(Length::Fill).height(Length::Fill);

        container(canvas_widget)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Color::BLACK.into()),
                ..Default::default()
            })
            .into()
    }
}

impl<Message> canvas::Program<Message> for EyeProtect {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let text_geometry = self.text.draw(renderer, bounds.size(), |frame| {
            let center = frame.center();

            // 修正：iced 0.13+ 的 Text 欄位名與 alignment 類型
            frame.fill_text(canvas::Text {
                content: format!("{:02} s", self.value),
                position: center,
                color: Color::WHITE,
                size: Pixels(150.0),
                align_x: alignment::Horizontal::Center.into(),
                align_y: alignment::Vertical::Center,
                ..Default::default()
            });

            if let Some(remind) = &self.remind {
                frame.fill_text(canvas::Text {
                    content: remind.clone(),
                    position: [center.x, center.y / 3.0].into(),
                    color: Color::WHITE,
                    size: Pixels(50.0),
                    align_x: alignment::Horizontal::Center.into(),
                    align_y: alignment::Vertical::Top,
                    ..Default::default()
                });
            }
        });

        vec![text_geometry]
    }
}
