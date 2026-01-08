//! Domain model for the application state

use iced::window;
use crate::providers::TTSProvider;
use crate::config;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TTSBackend {
    Piper,
    AwsPolly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

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
    ProviderSelected(TTSBackend),
    LogLevelSelected(LogLevel),
    TextCleanupToggled(bool),
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    TTSInitialized(Result<(), String>), // Result of async TTS initialization
    SelectedTextFetched(Option<String>), // Result of async text selection fetch
    TextCleanupResponse(Result<String, String>), // Result of text cleanup API call
}

/// Application state.
///
/// Note: Does not derive `Clone` because the TTS provider contains
/// audio resources that cannot be cloned.
pub struct App {
    pub playback_state: PlaybackState,
    pub progress: f32,
    pub frequency_bands: Vec<f32>,
    pub provider: Option<Box<dyn TTSProvider>>,
    pub selected_backend: TTSBackend,
    pub log_level: LogLevel,
    pub text_cleanup_enabled: bool,
    pub show_settings_modal: bool,
    pub settings_window_id: Option<window::Id>,
    pub current_window_id: Option<window::Id>,
    pub main_window_id: Option<window::Id>,
    pub pending_text: Option<String>,
    pub error_message: Option<String>,
    pub is_loading: bool,
    pub loading_animation_time: f32,
    /// Status text shown during loading (e.g., "Cleaning text...", "Synthesizing voice...")
    pub status_text: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            playback_state: PlaybackState::Stopped,
            progress: 0.0,
            frequency_bands: vec![0.0; 10],
            provider: None,
            selected_backend: TTSBackend::Piper,
            log_level: LogLevel::Info,
            text_cleanup_enabled: false,
            show_settings_modal: false,
            settings_window_id: None,
            current_window_id: None,
            main_window_id: None,
            pending_text: None,
            error_message: None,
            is_loading: false,
            loading_animation_time: 0.0,
            status_text: None,
        }
    }
}

impl App {
    /// Create a new app with pending text to speak.
    pub fn new(pending_text: Option<String>) -> Self {
        let selected_backend = config::load_voice_provider();
        let log_level = config::load_log_level();
        let text_cleanup_enabled = config::load_text_cleanup_enabled();
        Self {
            playback_state: PlaybackState::Stopped,
            progress: 0.0,
            frequency_bands: vec![0.0; 10],
            provider: None,
            selected_backend,
            log_level,
            text_cleanup_enabled,
            show_settings_modal: false,
            settings_window_id: None,
            current_window_id: None,
            main_window_id: None,
            pending_text,
            error_message: None,
            is_loading: false,
            loading_animation_time: 0.0,
            status_text: None,
        }
    }
}
