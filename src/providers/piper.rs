//! Piper TTS provider implementation.
//!
//! Uses the Piper binary to synthesize speech from text and plays it using rodio.

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use tracing::{debug, error, info, warn};

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
    /// 1. Project root: `./venv/bin/piper` (development)
    /// 2. User installation: `~/.local/share/grars/venv/bin/piper` (XDG Base Directory)
    /// 3. System PATH
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

        // Validate that the binary and model actually exist before continuing.
        if !piper_bin.is_file() {
            error!(?piper_bin, "Piper binary not found");
            return Err(TTSError::ProcessError(format!(
                "Piper binary not found at {}",
                piper_bin.display()
            )));
        }
        if !model_with_extension(&model_path).is_file() {
            error!(?model_path, "Piper model file (.onnx) not found");
            return Err(TTSError::ProcessError(format!(
                "Piper model (.onnx) not found at {}",
                model_with_extension(&model_path).display()
            )));
        }

        // Piper uses 22050 Hz sample rate
        let player = AudioPlayer::new(22050)?;

        Ok(Self {
            piper_bin,
            model_path,
            player,
        })
    }

    /// On macOS, check Linux-style path (~/.local/share/grars) for compatibility.
    #[cfg(target_os = "macos")]
    fn check_linux_style_path(relative_path: &str) -> Option<PathBuf> {
        dirs::home_dir().map(|home| {
            home.join(".local")
                .join("share")
                .join("grars")
                .join(relative_path)
        })
    }

    /// Find the piper binary in standard locations.
    fn find_piper_binary() -> PathBuf {
        // Check project-local virtualenv first (development)
        if let Ok(current_dir) = env::current_dir() {
            let project_piper = current_dir.join("venv").join("bin").join("piper");
            if project_piper.exists() {
                debug!(
                    path = %project_piper.display(),
                    "Using project-local piper binary"
                );
                return project_piper;
            }
        }

        // Check user installation (XDG Base Directory standard: ~/.local/share/grars)
        if let Some(data_dir) = dirs::data_dir() {
            let user_piper = data_dir.join("grars").join("venv").join("bin").join("piper");
            if user_piper.exists() {
                debug!(path = %user_piper.display(), "Using user-installed piper binary");
                return user_piper;
            }
        }

        // On macOS, also check Linux-style location (~/.local/share/grars)
        // since install scripts may use this location
        #[cfg(target_os = "macos")]
        {
            if let Some(linux_style_piper) = Self::check_linux_style_path("venv/bin/piper") {
                if linux_style_piper.exists() {
                    debug!(path = %linux_style_piper.display(), "Using Linux-style piper binary (macOS)");
                    return linux_style_piper;
                }
            }
        }

        // Check system PATH
        if let Ok(output) = Command::new("which").arg("piper").output() {
            if output.status.success() {
                if let Ok(path_str) = String::from_utf8(output.stdout) {
                    let trimmed = path_str.trim();
                    if !trimmed.is_empty() {
                        let path_buf = PathBuf::from(trimmed);
                        debug!(path = %path_buf.display(), "Using piper from PATH");
                        return path_buf;
                    }
                }
            }
        }

        // Fallback to user location (will fail validation)
        let fallback = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("grars")
            .join("venv")
            .join("bin")
            .join("piper");
        warn!(
            path = %fallback.display(),
            "Piper binary not found in known locations, using fallback path"
        );
        fallback
    }

    /// Find the model file in standard locations.
    fn find_model() -> PathBuf {
        let model_name = "en_US-lessac-medium";

        // Check project models directory first (for development)
        if let Ok(current_dir) = env::current_dir() {
            let project_model = current_dir.join("models").join(model_name);
            if project_model.with_extension("onnx").exists() {
                debug!(
                    path = %project_model.with_extension("onnx").display(),
                    "Using project Piper model"
                );
                return project_model;
            }
        }

        // Check user installation (XDG Base Directory standard: ~/.local/share/grars)
        if let Some(data_dir) = dirs::data_dir() {
            let user_model = data_dir.join("grars").join("models").join(model_name);
            if user_model.with_extension("onnx").exists() {
                debug!(
                    path = %user_model.with_extension("onnx").display(),
                    "Using user-installed Piper model"
                );
                return user_model;
            }
        }

        // On macOS, also check Linux-style location (~/.local/share/grars)
        // since install scripts may use this location
        #[cfg(target_os = "macos")]
        {
            if let Some(linux_style_model) = Self::check_linux_style_path(&format!("models/{}", model_name)) {
                if linux_style_model.with_extension("onnx").exists() {
                    debug!(
                        path = %linux_style_model.with_extension("onnx").display(),
                        "Using Linux-style Piper model (macOS)"
                    );
                    return linux_style_model;
                }
            }
        }

        // Fallback to user location (will fail validation)
        let fallback = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("grars")
            .join("models")
            .join(model_name);
        warn!(
            path = %fallback.with_extension("onnx").display(),
            "Piper model not found in known locations, using fallback path"
        );
        fallback
    }
}

/// Helper to get the model path including the `.onnx` extension.
fn model_with_extension(path: &Path) -> PathBuf {
    path.with_extension("onnx")
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
