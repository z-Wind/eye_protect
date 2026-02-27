use clap::Parser;
use eye_protect::GuiArgs;
use iced::{
    Color, Element, Length, Pixels, Rectangle, Renderer, Subscription, Task, Theme, alignment,
    keyboard,
    widget::{
        canvas,
        canvas::{Cache, Geometry},
        container,
    },
    window,
};

pub fn main() -> iced::Result {
    let args = GuiArgs::parse();

    let config = Config {
        top_enable: args.top_enable,
        wait_seconds: args.wait_seconds,
        remind: args.remind,
    };

    let window_level = if config.top_enable {
        window::Level::AlwaysOnTop
    } else {
        window::Level::Normal
    };

    iced::application(
        move || {
            let task =
                window::latest().and_then(|id| window::set_mode(id, window::Mode::Fullscreen));
            (EyeProtect::new(config.clone()), task)
        },
        EyeProtect::update,
        EyeProtect::view,
    )
    .window(window::Settings {
        level: window_level,
        ..Default::default()
    })
    .title("Eye Protect")
    .subscription(EyeProtect::subscription)
    .theme(|_: &EyeProtect| Theme::Dark)
    .run()
}

// ── Config（靜態初始化參數）────────────────────────────────────────────────────

#[derive(Clone)]
struct Config {
    top_enable: bool,
    wait_seconds: u32,
    remind: Option<String>,
}

// ── State（動態運行狀態）──────────────────────────────────────────────────────

struct EyeProtect {
    config: Config,
    remaining: u32,
    /// 倒數數字的 cache（每秒清除一次）
    timer_cache: Cache,
    /// 提醒文字的 cache（靜態，只繪製一次）
    remind_cache: Cache,
}

impl EyeProtect {
    fn new(config: Config) -> Self {
        Self {
            remaining: config.wait_seconds,
            timer_cache: Cache::default(),
            remind_cache: Cache::default(),
            config,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    KeyPressed(keyboard::Key),
}

// ── Update ───────────────────────────────────────────────────────────────────

impl EyeProtect {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.timer_cache.clear();

                // #1 用 checked_sub 搭配 u32，避免下溢；== 0 時關閉
                match self.remaining.checked_sub(1) {
                    Some(0) | None => {
                        return window::latest().and_then(window::close);
                    }
                    Some(r) => self.remaining = r,
                }
            }
            // #11 ESC 以結束碼 1 退出，表示使用者主動跳過
            Message::KeyPressed(keyboard::Key::Named(keyboard::key::Named::Escape)) => {
                return window::latest().and_then(window::close);
            }
            _ => {}
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick),
            iced::event::listen_with(|event, _status, _id| match event {
                iced::Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                    Some(Message::KeyPressed(key))
                }
                _ => None,
            }),
        ])
    }

    fn view(&self) -> Element<'_, Message> {
        container(canvas(self).width(Length::Fill).height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Color::BLACK.into()),
                ..Default::default()
            })
            .into()
    }
}

// ── Canvas ───────────────────────────────────────────────────────────────────

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
        let center = bounds.center();

        // 倒數計時（每秒重繪）
        let timer = self.timer_cache.draw(renderer, bounds.size(), |frame| {
            frame.fill_text(canvas::Text {
                content: format!("{:02} s", self.remaining),
                position: center,
                color: timer_color(self.remaining),
                size: Pixels(120.0),
                align_x: alignment::Horizontal::Center.into(),
                align_y: alignment::Vertical::Center,
                ..Default::default()
            });
        });

        // #2 只有在有提醒文字時才產生 remind geometry
        if self.config.remind.is_none() {
            return vec![timer];
        }

        // 提醒文字（靜態，只繪製一次，位置固定在倒數數字上方 110px）
        let remind = self.remind_cache.draw(renderer, bounds.size(), |frame| {
            if let Some(text) = &self.config.remind {
                frame.fill_text(canvas::Text {
                    content: text.clone(),
                    position: [center.x, center.y - 110.0].into(),
                    color: Color::from_rgb(0.72, 0.78, 0.88), // 淡藍灰，與各階段倒數顏色皆協調
                    size: Pixels(72.0),
                    align_x: alignment::Horizontal::Center.into(),
                    align_y: alignment::Vertical::Center,
                    ..Default::default()
                });
            }
        });

        vec![remind, timer]
    }
}

/// 根據剩餘秒數動態改變倒數顏色，使用低飽和護眼色系：
///  \>= 10s  柔和綠白（放鬆）
///  5–10s  淡琥珀黃（提醒）
///  <= 5s   霧橘（警示，但不刺眼）
fn timer_color(remaining: u32) -> Color {
    match remaining {
        r if r > 10 => Color::from_rgb(0.78, 0.93, 0.82), // 柔和綠白
        r if r > 5 => Color::from_rgb(0.93, 0.85, 0.60),  // 淡琥珀黃
        _ => Color::from_rgb(0.93, 0.65, 0.45),           // 霧橘
    }
}
