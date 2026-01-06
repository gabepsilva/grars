//! Shared audio playback infrastructure for TTS providers.
//!
//! Extracts common playback logic (rodio sink, position tracking, FFT visualization)
//! so providers only need to implement audio synthesis.

use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::thread;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use rustfft::{num_complex::Complex, FftPlanner};
use tracing::{debug, error, trace};

use super::TTSError;

/// Internal playback state shared between threads.
#[derive(Default)]
pub struct PlaybackState {
    /// Audio samples (normalized f32, -1.0 to 1.0)
    pub audio_data: Vec<f32>,
    /// Current playback position in samples
    pub position: usize,
    /// Whether playback is active
    pub is_playing: bool,
    /// Whether playback is paused
    pub is_paused: bool,
    /// Recent audio chunk for FFT visualization
    pub current_chunk: Vec<f32>,
}

/// Shared audio playback engine for TTS providers.
///
/// Handles rodio output, position tracking, and FFT visualization.
/// Providers compose with this struct and call `play_audio()` after synthesis.
pub struct AudioPlayer {
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

impl AudioPlayer {
    /// Create a new audio player with the given sample rate.
    pub fn new(sample_rate: u32) -> Result<Self, TTSError> {
        trace!(sample_rate, "AudioPlayer::new");
        let (stream, stream_handle) = OutputStream::try_default().map_err(|e| {
            error!("Failed to open audio output: {e}");
            TTSError::AudioError(format!("Failed to open audio output: {e}"))
        })?;

        debug!(sample_rate, "Audio output stream initialized");

        Ok(Self {
            sample_rate,
            state: Arc::new(Mutex::new(PlaybackState::default())),
            _stream: Some(stream),
            stream_handle: Some(stream_handle),
            sink: None,
        })
    }

    /// Load audio data and start playback.
    ///
    /// Call this after synthesizing audio. The audio_data should be normalized
    /// f32 samples in the range -1.0 to 1.0.
    pub fn play_audio(&mut self, audio_data: Vec<f32>) -> Result<(), TTSError> {
        debug!(samples = audio_data.len(), "AudioPlayer::play_audio");
        // Store audio data
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

    /// Convert raw PCM bytes (16-bit signed LE mono) to normalized f32 samples.
    pub fn pcm_to_f32(pcm_bytes: &[u8]) -> Vec<f32> {
        pcm_bytes
            .chunks_exact(2)
            .map(|chunk| {
                let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                sample as f32 / 32768.0
            })
            .collect()
    }

    /// Pause the current playback.
    pub fn pause(&mut self) -> Result<(), TTSError> {
        trace!("AudioPlayer::pause");
        if let Some(ref sink) = self.sink {
            sink.pause();
        }

        let mut state = self.state.lock().unwrap();
        if state.is_playing && !state.is_paused {
            state.is_paused = true;
        }
        Ok(())
    }

    /// Resume paused playback.
    pub fn resume(&mut self) -> Result<(), TTSError> {
        trace!("AudioPlayer::resume");
        if let Some(ref sink) = self.sink {
            sink.play();
        }

        let mut state = self.state.lock().unwrap();
        if state.is_paused {
            state.is_paused = false;
        }
        Ok(())
    }

    /// Stop playback and reset position.
    pub fn stop(&mut self) -> Result<(), TTSError> {
        trace!("AudioPlayer::stop");
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

    /// Check if audio is currently playing.
    pub fn is_playing(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.is_playing && !state.is_paused
    }

    /// Check if audio is currently paused.
    pub fn is_paused(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.is_paused
    }

    /// Skip forward by the given number of seconds.
    pub fn skip_forward(&mut self, seconds: f32) {
        trace!(seconds, "AudioPlayer::skip_forward");
        let samples_to_skip = (seconds * self.sample_rate as f32) as usize;
        let new_position = {
            let state = self.state.lock().unwrap();
            (state.position + samples_to_skip).min(state.audio_data.len())
        };
        self.seek_to(new_position).ok();
    }

    /// Skip backward by the given number of seconds.
    pub fn skip_backward(&mut self, seconds: f32) {
        trace!(seconds, "AudioPlayer::skip_backward");
        let samples_to_skip = (seconds * self.sample_rate as f32) as usize;
        let new_position = {
            let state = self.state.lock().unwrap();
            state.position.saturating_sub(samples_to_skip)
        };
        self.seek_to(new_position).ok();
    }

    /// Get playback progress as a value between 0.0 and 1.0.
    pub fn get_progress(&self) -> f32 {
        let state = self.state.lock().unwrap();
        if state.audio_data.is_empty() {
            return 0.0;
        }
        (state.position as f32 / state.audio_data.len() as f32).clamp(0.0, 1.0)
    }

    /// Get frequency band amplitudes for audio visualization.
    pub fn get_frequency_bands(&self, num_bands: usize) -> Vec<f32> {
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
        let magnitudes: Vec<f32> = buffer[..half_n].iter().map(|c| c.norm()).collect();

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

    /// Start audio playback from current position.
    fn start_playback(&mut self) -> Result<(), TTSError> {
        trace!("AudioPlayer::start_playback");
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
        let source = Decoder::new(cursor).map_err(|e| {
            error!("Failed to decode audio: {e}");
            TTSError::AudioError(format!("Failed to decode audio: {e}"))
        })?;

        let sink = Sink::try_new(stream_handle).map_err(|e| {
            error!("Failed to create audio sink: {e}");
            TTSError::AudioError(format!("Failed to create audio sink: {e}"))
        })?;

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
        trace!(
            samples = samples.len(),
            sample_rate,
            "AudioPlayer::create_wav"
        );
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
        trace!(
            start_position,
            sample_rate = self.sample_rate,
            "AudioPlayer::start_position_tracker_from"
        );
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
        trace!(position, "AudioPlayer::seek_to");
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

        // Give the old tracker thread time to exit (it checks is_playing every 75ms)
        if was_playing {
            thread::sleep(std::time::Duration::from_millis(80));
        }

        // Restart playback if we were playing
        if was_playing {
            self.start_playback()?;
        }

        Ok(())
    }
}

