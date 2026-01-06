//! Piper TTS provider implementation.
//!
//! Uses the Piper binary to synthesize speech from text and plays it using rodio.

use std::io::Cursor;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use rustfft::{num_complex::Complex, FftPlanner};

use super::{TTSError, TTSProvider};

/// Internal playback state shared between threads.
#[derive(Default)]
struct PlaybackState {
    /// Audio samples (normalized f32, -1.0 to 1.0)
    audio_data: Vec<f32>,
    /// Current playback position in samples
    position: usize,
    /// Whether playback is active
    is_playing: bool,
    /// Whether playback is paused
    is_paused: bool,
    /// Recent audio chunk for FFT visualization
    current_chunk: Vec<f32>,
}

/// Piper TTS provider using local ONNX models.
pub struct PiperTTSProvider {
    /// Path to the piper binary
    piper_bin: PathBuf,
    /// Path to the model file (without .onnx extension)
    model_path: PathBuf,
    /// Sample rate for audio output
    sample_rate: u32,
    /// Thread-safe playback state
    state: Arc<Mutex<PlaybackState>>,
    /// Audio output stream (must be kept alive)
    _stream: Option<OutputStream>,
    /// Audio output stream handle
    stream_handle: Option<OutputStreamHandle>,
    /// Audio sink for playback control
    sink: Option<Sink>,
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

        // Initialize audio output
        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| TTSError::AudioError(format!("Failed to open audio output: {e}")))?;

        Ok(Self {
            piper_bin,
            model_path,
            sample_rate: 22050,
            state: Arc::new(Mutex::new(PlaybackState::default())),
            _stream: Some(stream),
            stream_handle: Some(stream_handle),
            sink: None,
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

        // Check user installation
        if let Some(data_dir) = dirs::data_dir() {
            let user_model = data_dir.join("grars").join("models").join(model_name);
            if user_model.with_extension("onnx").exists() {
                return user_model;
            }
        }

        // Check dad project (grafl) for development
        if let Some(home) = dirs::home_dir() {
            let grafl_model = home.join("git_projects").join("grafl").join(model_name);
            if grafl_model.with_extension("onnx").exists() {
                return grafl_model;
            }
        }

        // Fallback to user location (will fail validation)
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("grars")
            .join("models")
            .join(model_name)
    }

    /// Convert raw PCM bytes (16-bit signed LE mono) to normalized f32 samples.
    fn pcm_to_f32(pcm_bytes: &[u8]) -> Vec<f32> {
        pcm_bytes
            .chunks_exact(2)
            .map(|chunk| {
                let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                sample as f32 / 32768.0
            })
            .collect()
    }

    /// Start audio playback from current position.
    fn start_playback(&mut self) -> Result<(), TTSError> {
        // Stop any existing playback first
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }

        let stream_handle = self
            .stream_handle
            .as_ref()
            .ok_or_else(|| TTSError::AudioError("No audio output available".into()))?;

        // Get audio data from current position
        let (audio_slice, position) = {
            let state = self.state.lock().unwrap();
            if state.audio_data.is_empty() {
                return Err(TTSError::AudioError("No audio data to play".into()));
            }
            let pos = state.position.min(state.audio_data.len());
            if pos >= state.audio_data.len() {
                return Err(TTSError::AudioError("Playback position at end".into()));
            }
            (state.audio_data[pos..].to_vec(), pos)
        };

        // Convert f32 samples back to i16 for WAV encoding
        let samples_i16: Vec<i16> = audio_slice
            .iter()
            .map(|&s| (s * 32767.0).clamp(-32768.0, 32767.0) as i16)
            .collect();

        // Create a WAV in memory
        let wav_data = Self::create_wav(&samples_i16, self.sample_rate);

        // Create decoder and sink
        let cursor = Cursor::new(wav_data);
        let source = Decoder::new(cursor)
            .map_err(|e| TTSError::AudioError(format!("Failed to decode audio: {e}")))?;

        let sink = Sink::try_new(stream_handle)
            .map_err(|e| TTSError::AudioError(format!("Failed to create audio sink: {e}")))?;

        sink.append(source);
        self.sink = Some(sink);

        // Update state
        {
            let mut state = self.state.lock().unwrap();
            state.is_playing = true;
            state.is_paused = false;
        }

        // Start position tracking in a background thread
        self.start_position_tracker_from(position);

        Ok(())
    }

    /// Create a WAV file in memory from i16 samples.
    fn create_wav(samples: &[i16], sample_rate: u32) -> Vec<u8> {
        let num_samples = samples.len();
        let data_size = num_samples * 2; // 16-bit = 2 bytes per sample
        let file_size = 36 + data_size;

        let mut wav = Vec::with_capacity(44 + data_size);

        // RIFF header
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(file_size as u32).to_le_bytes());
        wav.extend_from_slice(b"WAVE");

        // fmt chunk
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes()); // chunk size
        wav.extend_from_slice(&1u16.to_le_bytes()); // PCM format
        wav.extend_from_slice(&1u16.to_le_bytes()); // mono
        wav.extend_from_slice(&sample_rate.to_le_bytes());
        wav.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // byte rate
        wav.extend_from_slice(&2u16.to_le_bytes()); // block align
        wav.extend_from_slice(&16u16.to_le_bytes()); // bits per sample

        // data chunk
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&(data_size as u32).to_le_bytes());
        for &sample in samples {
            wav.extend_from_slice(&sample.to_le_bytes());
        }

        wav
    }

    /// Start a background thread to track playback position.
    fn start_position_tracker_from(&self, start_position: usize) {
        let state = Arc::clone(&self.state);
        let sample_rate = self.sample_rate;

        thread::spawn(move || {
            let chunk_duration_ms = 75; // Match UI update rate
            let samples_per_chunk = (sample_rate as usize * chunk_duration_ms) / 1000;

            // Initialize position to start position
            {
                let mut state_guard = state.lock().unwrap();
                state_guard.position = start_position;
            }

            loop {
                thread::sleep(std::time::Duration::from_millis(chunk_duration_ms as u64));

                let mut state_guard = state.lock().unwrap();

                // Exit thread if not playing (stopped or position changed externally)
                if !state_guard.is_playing {
                    break;
                }

                if state_guard.is_paused {
                    continue;
                }

                // Update position
                let new_position = state_guard.position + samples_per_chunk;
                if new_position >= state_guard.audio_data.len() {
                    state_guard.is_playing = false;
                    state_guard.position = state_guard.audio_data.len();
                    break;
                }

                state_guard.position = new_position;

                // Store current chunk for visualization
                let start = new_position.saturating_sub(samples_per_chunk);
                let end = new_position.min(state_guard.audio_data.len());
                state_guard.current_chunk = state_guard.audio_data[start..end].to_vec();
            }
        });
    }

    /// Seek to a new position and restart playback.
    fn seek_to(&mut self, position: usize) -> Result<(), TTSError> {
        let was_playing = {
            let state = self.state.lock().unwrap();
            state.is_playing && !state.is_paused
        };

        // Update position in state
        {
            let mut state = self.state.lock().unwrap();
            state.position = position.min(state.audio_data.len());
            state.is_playing = false; // Stop current tracker thread
        }

        // Restart playback if we were playing
        if was_playing {
            self.start_playback()?;
        }

        Ok(())
    }
}

impl TTSProvider for PiperTTSProvider {
    fn speak(&mut self, text: &str) -> Result<(), TTSError> {
        // Stop any current playback
        self.stop()?;

        // Run piper to generate audio
        let mut child = Command::new(&self.piper_bin)
            .args(["--model", self.model_path.to_str().unwrap_or(""), "--output_file", "-"])
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

        // Convert PCM to f32 and store
        let audio_data = Self::pcm_to_f32(&output.stdout);

        {
            let mut state = self.state.lock().unwrap();
            state.audio_data = audio_data;
            state.position = 0;
            state.is_playing = false;
            state.is_paused = false;
            state.current_chunk.clear();
        }

        // Start playback
        self.start_playback()
    }

    fn pause(&mut self) -> Result<(), TTSError> {
        if let Some(ref sink) = self.sink {
            sink.pause();
        }

        let mut state = self.state.lock().unwrap();
        if state.is_playing && !state.is_paused {
            state.is_paused = true;
        }
        Ok(())
    }

    fn resume(&mut self) -> Result<(), TTSError> {
        if let Some(ref sink) = self.sink {
            sink.play();
        }

        let mut state = self.state.lock().unwrap();
        if state.is_paused {
            state.is_paused = false;
        }
        Ok(())
    }

    fn stop(&mut self) -> Result<(), TTSError> {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }

        let mut state = self.state.lock().unwrap();
        state.is_playing = false;
        state.is_paused = false;
        state.position = 0;
        state.current_chunk.clear();
        Ok(())
    }

    fn is_playing(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.is_playing && !state.is_paused
    }

    fn is_paused(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.is_paused
    }

    fn skip_forward(&mut self, seconds: f32) {
        let samples_to_skip = (seconds * self.sample_rate as f32) as usize;
        let new_position = {
            let state = self.state.lock().unwrap();
            (state.position + samples_to_skip).min(state.audio_data.len())
        };
        self.seek_to(new_position).ok();
    }

    fn skip_backward(&mut self, seconds: f32) {
        let samples_to_skip = (seconds * self.sample_rate as f32) as usize;
        let new_position = {
            let state = self.state.lock().unwrap();
            state.position.saturating_sub(samples_to_skip)
        };
        self.seek_to(new_position).ok();
    }

    fn get_progress(&self) -> f32 {
        let state = self.state.lock().unwrap();
        if state.audio_data.is_empty() {
            return 0.0;
        }
        (state.position as f32 / state.audio_data.len() as f32).clamp(0.0, 1.0)
    }

    fn get_frequency_bands(&self, num_bands: usize) -> Vec<f32> {
        let state = self.state.lock().unwrap();

        if state.current_chunk.len() < 128 {
            return vec![0.0; num_bands];
        }

        let chunk = state.current_chunk.clone();
        drop(state); // Release lock before FFT computation

        // Apply Hanning window
        let n = chunk.len();
        let windowed: Vec<Complex<f32>> = chunk
            .iter()
            .enumerate()
            .map(|(i, &sample)| {
                let window = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / n as f32).cos());
                Complex::new(sample * window, 0.0)
            })
            .collect();

        // Perform FFT
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(n);
        let mut buffer = windowed;
        fft.process(&mut buffer);

        // Get magnitude of positive frequencies only
        let half_n = n / 2;
        let magnitudes: Vec<f32> = buffer[..half_n]
            .iter()
            .map(|c| c.norm())
            .collect();

        if magnitudes.len() < num_bands {
            return vec![0.0; num_bands];
        }

        // Split into logarithmic frequency bands
        let mut bands = Vec::with_capacity(num_bands);
        let log_max = (magnitudes.len() as f32).log10();

        for i in 0..num_bands {
            let start = (10f32.powf(log_max * i as f32 / num_bands as f32)) as usize;
            let end = (10f32.powf(log_max * (i + 1) as f32 / num_bands as f32)) as usize;
            let end = end.min(magnitudes.len());

            if end > start {
                // Use RMS for better energy representation
                let sum_sq: f32 = magnitudes[start..end].iter().map(|&x| x * x).sum();
                let rms = (sum_sq / (end - start) as f32).sqrt();
                bands.push(rms);
            } else {
                bands.push(0.0);
            }
        }

        // Normalize and apply power curve
        let max_val = bands.iter().cloned().fold(0.0f32, f32::max);
        if max_val > 0.0 {
            for band in &mut bands {
                *band = (*band / max_val).powf(0.7);
            }
        }

        bands
    }

}

