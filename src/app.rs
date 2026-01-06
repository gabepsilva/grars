//! Iced application adapter (thin UI layer)

use iced::time::{self, Duration};
use iced::{Application, Command, Element, Subscription, Theme};

use crate::model::{App, Message, PlaybackState};
use crate::providers::PiperTTSProvider;
use crate::update;
use crate::view;

/// Flags passed to the application at startup.
#[derive(Default)]
pub struct AppFlags {
    /// The TTS provider (already speaking if text was provided)
    pub provider: Option<PiperTTSProvider>,
}

impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = AppFlags;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        (Self::new(flags.provider), Command::none())
    }

    fn title(&self) -> String {
        String::from("Speaking...")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        update::update(self, message)
    }

    fn view(&self) -> Element<'_, Message> {
        view::view(self)
    }

    fn subscription(&self) -> Subscription<Message> {
        // Run animation/polling at ~75ms intervals
        // Only poll when playing (not stopped)
        match self.playback_state {
            PlaybackState::Stopped => Subscription::none(),
            _ => time::every(Duration::from_millis(75)).map(|_| Message::Tick),
        }
    }
}
