use iced::widget::{button, column, container, row, text, progress_bar, Space};
use iced::theme::{Button, Container};
use iced::{Alignment, Color, Element, Font, Length, Sandbox, Settings, Size};
use std::borrow::Cow;

// Noto Color Emoji font reference
const NOTO_EMOJI: Font = Font::with_name("Noto Color Emoji");

fn main() -> iced::Result {
    App::run(Settings {
        window: iced::window::Settings {
            size: Size::new(380.0, 70.0),
            resizable: false,
            decorations: false,
            transparent: true,
            ..Default::default()
        },
        fonts: vec![
            Cow::Borrowed(include_bytes!("../assets/NotoColorEmoji.ttf")),
        ],
        ..Default::default()
    })
}

#[derive(Debug, Clone, PartialEq)]
enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

struct App {
    playback_state: PlaybackState,
    progress: f32,
}

#[derive(Debug, Clone)]
enum Message {
    SkipBackward,
    SkipForward,
    PlayPause,
    Stop,
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> Self {
        Self {
            playback_state: PlaybackState::Playing,
            progress: 0.35,
        }
    }

    fn title(&self) -> String {
        String::from("Speaking...")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::SkipBackward => {
                self.progress = (self.progress - 0.1).max(0.0);
            }
            Message::SkipForward => {
                self.progress = (self.progress + 0.1).min(1.0);
            }
            Message::PlayPause => {
                self.playback_state = match self.playback_state {
                    PlaybackState::Playing => PlaybackState::Paused,
                    PlaybackState::Paused => PlaybackState::Playing,
                    PlaybackState::Stopped => PlaybackState::Playing,
                };
            }
            Message::Stop => {
                self.playback_state = PlaybackState::Stopped;
                self.progress = 0.0;
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // Speaker icon
        let speaker_icon = text("🔊")
            .size(24)
            .font(NOTO_EMOJI)
            .style(Color::WHITE);

        // Waveform visualization (10 bars represented with vertical lines)
        let wave_heights = [8, 16, 24, 20, 12, 18, 14, 22, 10, 16];
        let waveform: Element<Message> = row(
            wave_heights.iter().map(|&h| {
                container(Space::new(Length::Fixed(3.0), Length::Fixed(h as f32)))
                    .style(Container::Custom(Box::new(WaveBarStyle)))
                    .into()
            }).collect::<Vec<Element<Message>>>()
        )
        .spacing(4)
        .align_items(Alignment::Center)
        .into();

        // Control buttons
        let skip_back_btn = button(
            container(
                text("-5s")
                    .size(12)
                    .style(Color::WHITE)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
        )
        .width(Length::Fixed(36.0))
        .height(Length::Fixed(36.0))
        .style(Button::Custom(Box::new(CircleButtonStyle)))
        .on_press(Message::SkipBackward);

        let skip_fwd_btn = button(
            container(
                text("+5s")
                    .size(12)
                    .style(Color::WHITE)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
        )
        .width(Length::Fixed(36.0))
        .height(Length::Fixed(36.0))
        .style(Button::Custom(Box::new(CircleButtonStyle)))
        .on_press(Message::SkipForward);

        let play_pause_icon = match self.playback_state {
            PlaybackState::Playing => "⏸",
            _ => "▶",
        };
        
        let play_pause_btn = button(
            container(
                text(play_pause_icon)
                    .size(16)
                    .font(NOTO_EMOJI)
                    .style(Color::WHITE)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
        )
        .width(Length::Fixed(36.0))
        .height(Length::Fixed(36.0))
        .style(Button::Custom(Box::new(CircleButtonStyle)))
        .on_press(Message::PlayPause);

        let stop_btn = button(
            container(
                text("⏹")
                    .size(16)
                    .font(NOTO_EMOJI)
                    .style(Color::WHITE)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
        )
        .width(Length::Fixed(36.0))
        .height(Length::Fixed(36.0))
        .style(Button::Custom(Box::new(CircleButtonStyle)))
        .on_press(Message::Stop);

        let controls = row![
            skip_back_btn,
            skip_fwd_btn,
            play_pause_btn,
            stop_btn,
        ]
        .spacing(6)
        .align_items(Alignment::Center);

        // Settings icon
        let settings_icon = text("🔧")
            .size(16)
            .font(NOTO_EMOJI)
            .style(Color::from_rgba(1.0, 1.0, 1.0, 0.8));

        // Main horizontal bar
        let main_bar = row![
            speaker_icon,
            Space::with_width(Length::Fixed(16.0)),
            waveform,
            Space::with_width(Length::Fixed(16.0)),
            controls,
            Space::with_width(Length::Fixed(12.0)),
            settings_icon,
        ]
        .padding([8, 20, 4, 20])
        .align_items(Alignment::Center);

        // Progress bar at the bottom
        let progress = progress_bar(0.0..=1.0, self.progress)
            .height(Length::Fixed(2.0));

        let progress_container = container(progress)
            .padding([0, 27, 8, 49]);

        // Main content with progress bar below
        let content = column![
            main_bar,
            progress_container,
        ];

        // Window container with styling
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(Container::Custom(Box::new(WindowStyle)))
            .into()
    }
}

// Custom styles
struct WindowStyle;
impl container::StyleSheet for WindowStyle {
    type Style = iced::Theme;
    
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(iced::Background::Color(Color::BLACK)),
            border: iced::Border {
                color: Color::WHITE,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }
}

struct WaveBarStyle;
impl container::StyleSheet for WaveBarStyle {
    type Style = iced::Theme;
    
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.6))),
            border: iced::Border {
                radius: 1.5.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

struct CircleButtonStyle;
impl button::StyleSheet for CircleButtonStyle {
    type Style = iced::Theme;
    
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.15))),
            text_color: Color::WHITE,
            border: iced::Border {
                radius: 18.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
    
    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.25))),
            ..self.active(style)
        }
    }
    
    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.35))),
            ..self.active(style)
        }
    }
}

