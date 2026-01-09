//! Domain model for the application state

use std::collections::HashMap;
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
    StartDrag, // Begin dragging the window
    VoiceSelected(String), // Voice key selected (e.g., "en_US-lessac-medium")
    VoiceDownloadRequested(String), // Voice key to download
    VoiceDownloaded(Result<String, String>), // Download completion (voice key or error)
    VoicesJsonLoaded(Result<HashMap<String, VoiceInfo>, String>), // voices.json loaded
    OpenVoiceSelection(String), // Open voice selection window for language code
    CloseVoiceSelection, // Close voice selection window
}

/// Voice metadata from piper-voices repository
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct VoiceInfo {
    pub key: String,
    pub name: String,
    pub language: LanguageInfo,
    pub quality: String,
    pub num_speakers: u32,
    #[serde(default)]
    pub speaker_id_map: HashMap<String, u32>,
    pub files: HashMap<String, FileInfo>,
    #[serde(default)]
    pub aliases: Vec<String>,
}

/// Language information for a voice
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct LanguageInfo {
    pub code: String,
    pub family: String,
    pub region: String,
    pub name_native: String,
    pub name_english: String,
    pub country_english: String,
}

/// File information for voice model files
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct FileInfo {
    pub size_bytes: u64,
    pub md5_digest: String,
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
    /// Selected voice key (e.g., "en_US-lessac-medium")
    pub selected_voice: Option<String>,
    /// Selected language code for voice selection (e.g., "en_US")
    pub selected_language: Option<String>,
    /// All available voices loaded from voices.json
    pub voices: Option<HashMap<String, VoiceInfo>>,
    /// Voice selection window ID
    pub voice_selection_window_id: Option<window::Id>,
    /// Voice currently being downloaded (if any)
    pub downloading_voice: Option<String>,
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
            selected_voice: None,
            selected_language: None,
            voices: None,
            voice_selection_window_id: None,
            downloading_voice: None,
        }
    }
}

impl App {
    /// Create a new app with pending text to speak.
    pub fn new(pending_text: Option<String>) -> Self {
        let selected_backend = config::load_voice_provider();
        let log_level = config::load_log_level();
        let text_cleanup_enabled = config::load_text_cleanup_enabled();
        let selected_voice = config::load_selected_voice();
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
            selected_voice,
            selected_language: None,
            voices: None,
            voice_selection_window_id: None,
            downloading_voice: None,
        }
    }
}
