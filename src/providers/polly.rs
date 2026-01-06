//! AWS Polly TTS provider implementation.
//!
//! Uses the AWS SDK for Rust to synthesize speech and plays it using rodio.

use std::path::Path;

use aws_config::BehaviorVersion;
use aws_sdk_polly::types::{Engine, OutputFormat, VoiceId};

use super::audio_player::AudioPlayer;
use super::{TTSError, TTSProvider};

const DEFAULT_REGION: &str = "us-east-1";

/// AWS Polly TTS provider using the official AWS SDK.
pub struct PollyTTSProvider {
    /// AWS Polly client
    client: aws_sdk_polly::Client,
    /// Shared audio playback engine
    player: AudioPlayer,
    /// Tokio runtime for async AWS calls
    runtime: tokio::runtime::Runtime,
}

impl PollyTTSProvider {
    /// Create a new AWS Polly TTS provider.
    ///
    /// Loads credentials from `~/.aws/credentials` or environment variables.
    pub fn new() -> Result<Self, TTSError> {
        eprintln!("PollyTTSProvider: Initializing...");

        // Create a tokio runtime for async AWS SDK calls
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| TTSError::ProcessError(format!("Failed to create tokio runtime: {e}")))?;

        // Determine region: check ~/.aws/config, env vars, or default to us-east-1
        let region = Self::detect_region();
        eprintln!("PollyTTSProvider: Using region: {}", region);

        // Load AWS config (credentials from ~/.aws/credentials or env vars)
        let config = runtime.block_on(async {
            aws_config::defaults(BehaviorVersion::latest())
                .region(aws_config::Region::new(region.clone()))
                .load()
                .await
        });

        let client = aws_sdk_polly::Client::new(&config);
        eprintln!("PollyTTSProvider: AWS client created");

        // Polly neural voices use 16kHz sample rate
        let player = AudioPlayer::new(16000)?;

        Ok(Self {
            client,
            player,
            runtime,
        })
    }

    /// Detect AWS region from environment or config file.
    ///
    /// Priority:
    /// 1. AWS_REGION or AWS_DEFAULT_REGION environment variables
    /// 2. ~/.aws/config file (default profile)
    /// 3. Falls back to us-east-1
    fn detect_region() -> String {
        // Check environment variables first
        if let Ok(region) = std::env::var("AWS_REGION") {
            if !region.is_empty() {
                return region;
            }
        }
        if let Ok(region) = std::env::var("AWS_DEFAULT_REGION") {
            if !region.is_empty() {
                return region;
            }
        }

        // Check ~/.aws/config file
        if let Some(home) = dirs::home_dir() {
            let config_path = home.join(".aws").join("config");
            if let Some(region) = Self::read_region_from_config(&config_path) {
                return region;
            }
        }

        // Default to us-east-1
        DEFAULT_REGION.to_string()
    }

    /// Read region from AWS config file.
    fn read_region_from_config(path: &Path) -> Option<String> {
        let content = std::fs::read_to_string(path).ok()?;
        let profile = std::env::var("AWS_PROFILE").unwrap_or_else(|_| "default".to_string());

        // Look for [default] or [profile <name>] section
        let section_header = if profile == "default" {
            "[default]"
        } else {
            // For non-default profiles, AWS config uses [profile <name>]
            return Self::read_region_from_profile_section(&content, &profile);
        };

        let mut in_section = false;
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                in_section = line.eq_ignore_ascii_case(section_header);
                continue;
            }
            if in_section && line.starts_with("region") {
                if let Some(value) = line.split('=').nth(1) {
                    let region = value.trim();
                    if !region.is_empty() {
                        return Some(region.to_string());
                    }
                }
            }
        }
        None
    }

    /// Read region from a named profile section.
    fn read_region_from_profile_section(content: &str, profile: &str) -> Option<String> {
        let section_header = format!("[profile {}]", profile);
        let mut in_section = false;
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                in_section = line.eq_ignore_ascii_case(&section_header);
                continue;
            }
            if in_section && line.starts_with("region") {
                if let Some(value) = line.split('=').nth(1) {
                    let region = value.trim();
                    if !region.is_empty() {
                        return Some(region.to_string());
                    }
                }
            }
        }
        None
    }
}

impl TTSProvider for PollyTTSProvider {
    fn speak(&mut self, text: &str) -> Result<(), TTSError> {
        eprintln!("PollyTTSProvider: Speaking {} chars", text.len());

        // Stop any current playback
        self.player.stop()?;

        // Call AWS Polly to synthesize speech
        let audio_bytes = self.runtime.block_on(async {
            let response = self
                .client
                .synthesize_speech()
                .text(text)
                .output_format(OutputFormat::Pcm)
                .voice_id(VoiceId::Matthew)
                .engine(Engine::Neural)
                .sample_rate("16000")
                .send()
                .await
                .map_err(|e| TTSError::ProcessError(format!("AWS Polly API error: {e}")))?;

            let audio_stream = response.audio_stream;
            let bytes = audio_stream
                .collect()
                .await
                .map_err(|e| TTSError::ProcessError(format!("Failed to read audio stream: {e}")))?;

            Ok::<_, TTSError>(bytes.into_bytes().to_vec())
        })?;

        if audio_bytes.is_empty() {
            return Err(TTSError::ProcessError(
                "No audio data generated by AWS Polly".into(),
            ));
        }

        eprintln!(
            "PollyTTSProvider: Received {} bytes of audio",
            audio_bytes.len()
        );

        // Convert PCM to f32 and play
        let audio_data = AudioPlayer::pcm_to_f32(&audio_bytes);
        let duration_sec = audio_data.len() as f32 / 16000.0;
        eprintln!("PollyTTSProvider: Audio duration {:.1}s", duration_sec);

        self.player.play_audio(audio_data)
    }

    fn pause(&mut self) -> Result<(), TTSError> {
        self.player.pause()
    }

    fn resume(&mut self) -> Result<(), TTSError> {
        self.player.resume()
    }

    fn stop(&mut self) -> Result<(), TTSError> {
        self.player.stop()
    }

    fn is_playing(&self) -> bool {
        self.player.is_playing()
    }

    fn is_paused(&self) -> bool {
        self.player.is_paused()
    }

    fn skip_forward(&mut self, seconds: f32) {
        self.player.skip_forward(seconds);
    }

    fn skip_backward(&mut self, seconds: f32) {
        self.player.skip_backward(seconds);
    }

    fn get_progress(&self) -> f32 {
        self.player.get_progress()
    }

    fn get_frequency_bands(&self, num_bands: usize) -> Vec<f32> {
        self.player.get_frequency_bands(num_bands)
    }
}
