//! Piper TTS provider implementation.
//!
//! Uses the Piper binary to synthesize speech from text and plays it using rodio.

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

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
    /// 2. User installation: `~/.local/share/insight-reader/venv/bin/piper` (XDG Base Directory)
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

    /// On macOS, check Linux-style path (~/.local/share/insight-reader) for compatibility.
    #[cfg(target_os = "macos")]
    fn check_linux_style_path(relative_path: &str) -> Option<PathBuf> {
        dirs::home_dir().map(|home| {
            home.join(".local")
                .join("share")
                .join("insight-reader")
                .join(relative_path)
        })
    }

    /// Find the piper binary in standard locations.
    fn find_piper_binary() -> PathBuf {
        // Platform-specific paths for venv binaries
        #[cfg(target_os = "windows")]
        const VENV_BIN_DIR: &str = "Scripts";
        #[cfg(target_os = "windows")]
        const PIPER_BIN_NAME: &str = "piper.exe";
        
        #[cfg(not(target_os = "windows"))]
        const VENV_BIN_DIR: &str = "bin";
        #[cfg(not(target_os = "windows"))]
        const PIPER_BIN_NAME: &str = "piper";
        
        // Check project-local virtualenv first (development)
        if let Ok(current_dir) = env::current_dir() {
            let project_piper = current_dir.join("venv").join(VENV_BIN_DIR).join(PIPER_BIN_NAME);
            if project_piper.exists() {
                debug!(
                    path = %project_piper.display(),
                    "Using project-local piper binary"
                );
                return project_piper;
            }
        }

        // Check user installation
        // On Windows: %LOCALAPPDATA%\insight-reader\venv\Scripts\piper.exe
        // On Unix: ~/.local/share/insight-reader/venv/bin/piper
        if let Some(data_dir) = dirs::data_local_dir() {
            let user_piper = data_dir.join("insight-reader").join("venv").join(VENV_BIN_DIR).join(PIPER_BIN_NAME);
            if user_piper.exists() {
                debug!(path = %user_piper.display(), "Using user-installed piper binary (local data dir)");
                return user_piper;
            }
        }
        
        // Also check data_dir (XDG Base Directory standard on Unix)
        if let Some(data_dir) = dirs::data_dir() {
            let user_piper = data_dir.join("insight-reader").join("venv").join(VENV_BIN_DIR).join(PIPER_BIN_NAME);
            if user_piper.exists() {
                debug!(path = %user_piper.display(), "Using user-installed piper binary (data dir)");
                return user_piper;
            }
        }

        // On macOS, also check Linux-style location (~/.local/share/insight-reader)
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
        // On Windows use 'where', on Unix use 'which'
        #[cfg(target_os = "windows")]
        {
            let path_cmd = "where";
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            if let Ok(output) = Command::new(path_cmd)
                .arg("piper")
                .creation_flags(CREATE_NO_WINDOW)
                .output()
            {
                if output.status.success() {
                    if let Ok(path_str) = String::from_utf8(output.stdout) {
                        // 'where' on Windows may return multiple lines, take the first
                        let trimmed = path_str.lines().next().unwrap_or("").trim();
                        if !trimmed.is_empty() {
                            let path_buf = PathBuf::from(trimmed);
                            debug!(path = %path_buf.display(), "Using piper from PATH");
                            return path_buf;
                        }
                    }
                }
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let path_cmd = "which";
            if let Ok(output) = Command::new(path_cmd).arg("piper").output() {
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
        }

        // Fallback to user location (will fail validation)
        #[cfg(target_os = "windows")]
        let fallback_base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("C:\\Temp"));
        #[cfg(not(target_os = "windows"))]
        let fallback_base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        
        let fallback = fallback_base
            .join("insight-reader")
            .join("venv")
            .join(VENV_BIN_DIR)
            .join(PIPER_BIN_NAME);
        warn!(
            path = %fallback.display(),
            "Piper binary not found in known locations, using fallback path"
        );
        fallback
    }

    /// Find the model file in standard locations.
    fn find_model() -> PathBuf {
        // Try to load selected voice from config, fallback to default
        let model_name = crate::config::load_selected_voice()
            .unwrap_or_else(|| "en_US-lessac-medium".to_string());

        // Check project models directory first (for development)
        if let Ok(current_dir) = env::current_dir() {
            let project_model = current_dir.join("models").join(&model_name);
            if project_model.with_extension("onnx").exists() {
                debug!(
                    path = %project_model.with_extension("onnx").display(),
                    "Using project Piper model"
                );
                return project_model;
            }
        }

        // Check user installation
        // On Windows: %LOCALAPPDATA%\insight-reader\models
        // On Unix: ~/.local/share/insight-reader/models (via data_dir)
        if let Some(data_dir) = dirs::data_local_dir() {
            let user_model = data_dir.join("insight-reader").join("models").join(&model_name);
            if user_model.with_extension("onnx").exists() {
                debug!(
                    path = %user_model.with_extension("onnx").display(),
                    "Using user-installed Piper model (local data dir)"
                );
                return user_model;
            }
        }
        
        // Also check data_dir (XDG Base Directory standard on Unix)
        if let Some(data_dir) = dirs::data_dir() {
            let user_model = data_dir.join("insight-reader").join("models").join(&model_name);
            if user_model.with_extension("onnx").exists() {
                debug!(
                    path = %user_model.with_extension("onnx").display(),
                    "Using user-installed Piper model (data dir)"
                );
                return user_model;
            }
        }

        // On macOS, also check Linux-style location (~/.local/share/insight-reader)
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
        #[cfg(target_os = "windows")]
        let fallback_base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("C:\\Temp"));
        #[cfg(not(target_os = "windows"))]
        let fallback_base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        
        let fallback = fallback_base
            .join("insight-reader")
            .join("models")
            .join(&model_name);
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
        // Validate input text
        let text = text.trim();
        if text.is_empty() {
            warn!("Empty text provided to piper, skipping synthesis");
            return Err(TTSError::ProcessError(
                "Cannot synthesize empty text".into(),
            ));
        }

        debug!(
            chars = text.len(),
            text_preview = %text.chars().take(50).collect::<String>(),
            "Piper: synthesizing speech"
        );

        // Stop any current playback
        self.player.stop()?;

        // Build command for logging
        let model_arg = self.model_path.to_str().unwrap_or("");
        debug!(
            piper_bin = %self.piper_bin.display(),
            model_path = %model_arg,
            "Executing piper command"
        );

        // On Windows, piper has issues with stdout streaming, so we use a temp file
        // On Unix, we can stream directly to stdout for better performance
        #[cfg(target_os = "windows")]
        let audio_data = {
            use std::fs;
            use std::io::Write;
            
            // Create temp file for output
            let temp_dir = env::temp_dir();
            let temp_file = temp_dir.join("insight-reader-piper-output.wav");
            let temp_file_str = temp_file.to_string_lossy().to_string();
            
            debug!(temp_file = %temp_file_str, "Using temp file for piper output (Windows)");
            
            // Run piper with temp file output
            // Use CREATE_NO_WINDOW flag to prevent console window from appearing
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let mut child = Command::new(&self.piper_bin)
                .args([
                    "--model",
                    model_arg,
                    "--output_file",
                    &temp_file_str,
                ])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .creation_flags(CREATE_NO_WINDOW)
                .spawn()
                .map_err(|e| {
                    error!(
                        error = %e,
                        piper_bin = %self.piper_bin.display(),
                        "Failed to start piper process"
                    );
                    TTSError::ProcessError(format!("Failed to start piper: {e}"))
                })?;

            // Send text to piper
            {
                let stdin = child
                    .stdin
                    .as_mut()
                    .ok_or_else(|| TTSError::ProcessError("Failed to open piper stdin".into()))?;
                stdin
                    .write_all(text.as_bytes())
                    .map_err(|e| {
                        error!(error = %e, text_bytes = text.len(), "Failed to write text to piper stdin");
                        TTSError::ProcessError(format!("Failed to write to piper: {e}"))
                    })?;
                debug!(text_bytes = text.len(), "Text written to piper stdin");
            }

            // Wait for completion
            let output = child
                .wait_with_output()
                .map_err(|e| {
                    error!(error = %e, "Piper process wait failed");
                    TTSError::ProcessError(format!("Piper process failed: {e}"))
                })?;

            let exit_code = output.status.code();
            let stderr = String::from_utf8_lossy(&output.stderr);

            if !output.status.success() {
                error!(exit_code = ?exit_code, stderr = %stderr.trim(), "Piper process failed");
                // Clean up temp file on error
                let _ = fs::remove_file(&temp_file);
                return Err(TTSError::ProcessError(format!(
                    "Piper failed with code {:?}: {}",
                    exit_code,
                    stderr.trim()
                )));
            }

            // Read the WAV file and extract raw PCM data
            let wav_data = fs::read(&temp_file).map_err(|e| {
                error!(error = %e, path = %temp_file_str, "Failed to read piper output file");
                TTSError::ProcessError(format!("Failed to read piper output: {e}"))
            })?;
            
            // Clean up temp file
            let _ = fs::remove_file(&temp_file);
            
            if wav_data.is_empty() {
                error!("Piper produced empty output file");
                return Err(TTSError::ProcessError("No audio data generated by piper".into()));
            }
            
            // WAV files have a 44-byte header, skip it to get raw PCM
            // Verify it's a valid WAV file
            if wav_data.len() < 44 || &wav_data[0..4] != b"RIFF" {
                error!(bytes = wav_data.len(), "Invalid WAV file format from piper");
                return Err(TTSError::ProcessError("Invalid audio format from piper".into()));
            }
            
            let pcm_data = &wav_data[44..];
            AudioPlayer::pcm_to_f32(pcm_data)
        };
        
        #[cfg(not(target_os = "windows"))]
        let audio_data = {
            use std::io::Write;
            
            // Run piper to generate audio (stream to stdout)
            let mut child = Command::new(&self.piper_bin)
                .args([
                    "--model",
                    model_arg,
                    "--output_file",
                    "-",
                ])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| {
                    error!(
                        error = %e,
                        piper_bin = %self.piper_bin.display(),
                        "Failed to start piper process"
                    );
                    TTSError::ProcessError(format!("Failed to start piper: {e}"))
                })?;

            // Send text to piper
            {
                let stdin = child
                    .stdin
                    .as_mut()
                    .ok_or_else(|| TTSError::ProcessError("Failed to open piper stdin".into()))?;
                stdin
                    .write_all(text.as_bytes())
                    .map_err(|e| {
                        error!(
                            error = %e,
                            text_bytes = text.len(),
                            "Failed to write text to piper stdin"
                        );
                        TTSError::ProcessError(format!("Failed to write to piper: {e}"))
                    })?;
                debug!(text_bytes = text.len(), "Text written to piper stdin");
            }

            // Wait for completion and get output
            let output = child
                .wait_with_output()
                .map_err(|e| {
                    error!(error = %e, "Piper process wait failed");
                    TTSError::ProcessError(format!("Piper process failed: {e}"))
                })?;

            let exit_code = output.status.code();
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout_len = output.stdout.len();

            if !output.status.success() {
                error!(
                    exit_code = ?exit_code,
                    stderr = %stderr.trim(),
                    stdout_bytes = stdout_len,
                    "Piper process failed"
                );
                return Err(TTSError::ProcessError(format!(
                    "Piper failed with code {:?}: {}",
                    exit_code,
                    stderr.trim()
                )));
            }

            if output.stdout.is_empty() {
                // Log detailed diagnostics when no audio is generated
                error!(
                    exit_code = ?exit_code,
                    stderr = %stderr.trim(),
                    stdout_bytes = 0,
                    piper_bin = %self.piper_bin.display(),
                    model_path = %model_arg,
                    text_preview = %text.chars().take(100).collect::<String>(),
                    text_bytes = text.len(),
                    "Piper exited successfully but produced no audio output"
                );
                let error_msg = if stderr.trim().is_empty() {
                    "No audio data generated by piper".to_string()
                } else {
                    format!("No audio data generated by piper. stderr: {}", stderr.trim())
                };
                return Err(TTSError::ProcessError(error_msg));
            }

            // Convert PCM to f32
            AudioPlayer::pcm_to_f32(&output.stdout)
        };

        let duration_sec = audio_data.len() as f32 / 22050.0;
        info!(
            samples = audio_data.len(),
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
