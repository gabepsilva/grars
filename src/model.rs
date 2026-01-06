//! Domain model for the application state

use iced::window;
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
    Settings,
    CloseSettings,
    WindowOpened(window::Id),
    WindowClosed(window::Id),
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
    pub show_settings_modal: bool,
    pub settings_window_id: Option<window::Id>,
    pub current_window_id: Option<window::Id>,
    pub main_window_id: Option<window::Id>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            playback_state: PlaybackState::Stopped,
            progress: 0.0,
            frequency_bands: vec![0.0; 10],
            provider: None,
            show_settings_modal: false,
            settings_window_id: None,
            current_window_id: None,
            main_window_id: None,
        }
    }
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
            show_settings_modal: false,
            settings_window_id: None,
            current_window_id: None,
            main_window_id: None,
        }
    }
}
