//! TTS provider abstraction layer.
//!
//! This module defines the [`TTSProvider`] trait and provides implementations
//! for different TTS engines.

mod piper;

pub use piper::PiperTTSProvider;

use thiserror::Error;

/// Errors that can occur during TTS operations.
#[derive(Debug, Error)]
pub enum TTSError {
    #[error("Failed to start TTS process: {0}")]
    ProcessError(String),

    #[error("Audio playback error: {0}")]
    AudioError(String),

    #[error("Provider not configured: {0}")]
    ConfigError(String),

    #[error("Operation not supported: {0}")]
    NotSupported(String),
}

/// Abstract interface for TTS providers.
///
/// Allows plugging in different TTS engines (Piper, Polly, etc.).
pub trait TTSProvider {
    /// Speak the given text.
    ///
    /// This method synthesizes and plays the audio. It may block during
    /// synthesis but audio playback is non-blocking.
    fn speak(&mut self, text: &str) -> Result<(), TTSError>;

    /// Pause the current speech playback.
    fn pause(&mut self) -> Result<(), TTSError>;

    /// Resume paused speech playback.
    fn resume(&mut self) -> Result<(), TTSError>;

    /// Stop the current speech playback and reset position.
    fn stop(&mut self) -> Result<(), TTSError>;

    /// Check if speech is currently playing.
    fn is_playing(&self) -> bool;

    /// Check if speech is currently paused.
    fn is_paused(&self) -> bool;

    /// Skip forward in the current speech playback.
    fn skip_forward(&mut self, seconds: f32);

    /// Skip backward in the current speech playback.
    fn skip_backward(&mut self, seconds: f32);

    /// Get playback progress as a value between 0.0 and 1.0.
    fn get_progress(&self) -> f32;

    /// Get frequency band amplitudes for audio visualization.
    ///
    /// Returns normalized amplitude values (0.0-1.0) for each frequency band.
    fn get_frequency_bands(&self, num_bands: usize) -> Vec<f32>;

    /// Return the name of this TTS provider.
    fn name(&self) -> &str;

    /// Validate that the provider is properly configured and available.
    fn validate_config(&self) -> bool;
}
