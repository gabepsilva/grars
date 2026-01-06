//! Domain model for the application state

use crate::providers::PiperTTSProvider;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone)]
pub enum Message {
    SkipBackward,
    SkipForward,
    PlayPause,
    Stop,
    Tick,
}

/// Application state.
///
/// Note: Does not derive `Clone` because `PiperTTSProvider` contains
/// audio resources that cannot be cloned.
pub struct App {
    pub playback_state: PlaybackState,
    pub progress: f32,
    pub frequency_bands: Vec<f32>,
    pub provider: Option<PiperTTSProvider>,
}

impl App {
    /// Create a new app with the given TTS provider.
    pub fn new(provider: Option<PiperTTSProvider>) -> Self {
        Self {
            playback_state: provider
                .as_ref()
                .map_or(PlaybackState::Stopped, |_| PlaybackState::Playing),
            progress: 0.0,
            frequency_bands: vec![0.0; 10],
            provider,
        }
    }
}
