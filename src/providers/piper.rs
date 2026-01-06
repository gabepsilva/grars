//! Piper TTS provider implementation.
//!
//! Uses the Piper binary to synthesize speech from text and plays it using rodio.

use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use tracing::{debug, info};

use super::audio_player::AudioPlayer;
use super::{TTSError, TTSProvider};

/// Piper TTS provider using local ONNX models.
pub struct PiperTTSProvider {
    /// Path to the piper binary
    piper_bin: PathBuf,
    /// Path to the model file (without .onnx extension)
    model_path: PathBuf,
    /// Shared audio playback engine
    player: AudioPlayer,
}

impl PiperTTSProvider {
    /// Create a new Piper TTS provider with default configuration.
    ///
    /// Searches for piper binary and model in standard locations:
    /// 1. User installation: `~/.local/share/grars/`
    /// 2. System PATH
    pub fn new() -> Result<Self, TTSError> {
        Self::with_config(None, None)
    }

    /// Create a new Piper TTS provider with custom paths.
    ///
    /// # Arguments
    /// * `piper_bin` - Path to piper binary (None = auto-detect)
    /// * `model_path` - Path to model file without extension (None = auto-detect)
    pub fn with_config(
        piper_bin: Option<PathBuf>,
        model_path: Option<PathBuf>,
    ) -> Result<Self, TTSError> {
        let piper_bin = piper_bin.unwrap_or_else(Self::find_piper_binary);
        let model_path = model_path.unwrap_or_else(Self::find_model);

        info!("Initializing Piper TTS provider");
        debug!(?piper_bin, ?model_path, "Piper configuration");

        // Piper uses 22050 Hz sample rate
        let player = AudioPlayer::new(22050)?;

        Ok(Self {
            piper_bin,
            model_path,
            player,
        })
    }

    /// Find the piper binary in standard locations.
    fn find_piper_binary() -> PathBuf {
        // Check user installation first
        if let Some(data_dir) = dirs::data_dir() {
            let user_piper = data_dir.join("grars").join("venv").join("bin").join("piper");
            if user_piper.exists() {
                return user_piper;
            }
        }

        // Check dad project (grafl) venv for development
        if let Some(home) = dirs::home_dir() {
            let grafl_piper = home
                .join("git_projects")
                .join("grafl")
                .join("venv")
                .join("bin")
                .join("piper");
            if grafl_piper.exists() {
                return grafl_piper;
            }
        }

        // Check system PATH
        if let Ok(output) = Command::new("which").arg("piper").output() {
            if output.status.success() {
                if let Ok(path) = String::from_utf8(output.stdout) {
                    let path = path.trim();
                    if !path.is_empty() {
                        return PathBuf::from(path);
                    }
                }
            }
        }

        // Fallback to user location (will fail validation)
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("grars")
            .join("venv")
            .join("bin")
            .join("piper")
    }

    /// Find the model file in standard locations.
    fn find_model() -> PathBuf {
        let model_name = "en_US-lessac-medium";

        // Check project models directory first (for development)
        if let Ok(current_dir) = env::current_dir() {
            let project_model = current_dir.join("models").join(model_name);
            if project_model.with_extension("onnx").exists() {
                return project_model;
            }
        }

        // Check user installation
        if let Some(data_dir) = dirs::data_dir() {
            let user_model = data_dir.join("grars").join("models").join(model_name);
            if user_model.with_extension("onnx").exists() {
                return user_model;
            }
        }

        // Fallback to user location (will fail validation)
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("grars")
            .join("models")
            .join(model_name)
    }
}

impl TTSProvider for PiperTTSProvider {
    fn speak(&mut self, text: &str) -> Result<(), TTSError> {
        debug!(chars = text.len(), "Piper: synthesizing speech");

        // Stop any current playback
        self.player.stop()?;

        // Run piper to generate audio
        let mut child = Command::new(&self.piper_bin)
            .args([
                "--model",
                self.model_path.to_str().unwrap_or(""),
                "--output_file",
                "-",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| TTSError::ProcessError(format!("Failed to start piper: {e}")))?;

        // Send text to piper
        {
            use std::io::Write;
            let stdin = child
                .stdin
                .as_mut()
                .ok_or_else(|| TTSError::ProcessError("Failed to open piper stdin".into()))?;
            stdin
                .write_all(text.as_bytes())
                .map_err(|e| TTSError::ProcessError(format!("Failed to write to piper: {e}")))?;
        }

        // Wait for completion and get output
        let output = child
            .wait_with_output()
            .map_err(|e| TTSError::ProcessError(format!("Piper process failed: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TTSError::ProcessError(format!(
                "Piper failed with code {:?}: {}",
                output.status.code(),
                stderr
            )));
        }

        if output.stdout.is_empty() {
            return Err(TTSError::ProcessError(
                "No audio data generated by piper".into(),
            ));
        }

        // Convert PCM to f32 and play
        let audio_data = AudioPlayer::pcm_to_f32(&output.stdout);
        let duration_sec = audio_data.len() as f32 / 22050.0;
        info!(
            bytes = output.stdout.len(),
            duration_sec = format!("{:.1}", duration_sec),
            "Piper: audio generated"
        );

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
