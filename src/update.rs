//! Business logic for state transitions

use iced::window;
use iced::Command;

use crate::model::{App, Message, PlaybackState};
use crate::providers::TTSProvider;

const SKIP_SECONDS: f32 = 5.0;
const NUM_BANDS: usize = 10;

pub fn update(app: &mut App, message: Message) -> Command<Message> {
    match message {
        Message::SkipBackward => {
            if let Some(ref mut provider) = app.provider {
                provider.skip_backward(SKIP_SECONDS);
                // Position is updated synchronously in seek_to(), so progress is accurate immediately
                app.progress = provider.get_progress();
            }
            Command::none()
        }
        Message::SkipForward => {
            if let Some(ref mut provider) = app.provider {
                provider.skip_forward(SKIP_SECONDS);
                app.progress = provider.get_progress();
                // Don't close here - let Tick message handle it when playback actually finishes
            }
            Command::none()
        }
        Message::PlayPause => {
            if let Some(ref mut provider) = app.provider {
                match app.playback_state {
                    PlaybackState::Playing => {
                        if provider.pause().is_ok() {
                            app.playback_state = PlaybackState::Paused;
                        }
                    }
                    PlaybackState::Paused => {
                        if provider.resume().is_ok() {
                            app.playback_state = PlaybackState::Playing;
                        }
                    }
                    PlaybackState::Stopped => {
                        // Can't resume from stopped - would need to re-speak
                    }
                }
            }
            Command::none()
        }
        Message::Stop => {
            if let Some(ref mut provider) = app.provider {
                provider.stop().ok();
            }
            app.playback_state = PlaybackState::Stopped;
            app.progress = 0.0;
            app.frequency_bands = vec![0.0; NUM_BANDS];
            // Close window when stopped
            window::close(window::Id::MAIN)
        }
        Message::Tick => {
            if let Some(ref provider) = app.provider {
                // Update progress from provider
                app.progress = provider.get_progress();

                // Update frequency bands for visualization
                app.frequency_bands = provider.get_frequency_bands(NUM_BANDS);

                // Check if playback finished
                if !provider.is_playing() && !provider.is_paused() {
                    app.playback_state = PlaybackState::Stopped;
                    return window::close(window::Id::MAIN);
                }
            }
            Command::none()
        }
    }
}
