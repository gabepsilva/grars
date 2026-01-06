# grars Implementation Plan

Rewriting the original Python GTK4 TTS app in Rust with Iced.

## Current State

Basic Iced UI with:
- Floating borderless window
- Play/pause/stop buttons (UI only)
- Static waveform visualization
- Progress bar (hardcoded)
- Clipboard reading at startup

## Target Architecture

```
src/
├── main.rs              # Entry point
├── app.rs               # Iced Application (async-capable)
├── model.rs             # Domain types + app state
├── update.rs            # Business logic
├── view.rs              # UI rendering
├── styles.rs            # Custom styles
├── providers/
│   ├── mod.rs           # TTSProvider trait + factory
│   ├── piper.rs         # Piper TTS implementation
│   └── polly.rs         # (future) AWS Polly
├── system/
│   ├── mod.rs
│   └── clipboard.rs     # Selection/clipboard reading
└── config.rs            # Config management
```

---

## Phase 1: Core TTS Infrastructure

### 1.1 Create TTSProvider trait
- [x] Create `src/providers/mod.rs`
- [x] Define `TTSProvider` trait:
  ```rust
  pub trait TTSProvider: Send + Sync {
      fn speak(&mut self, text: &str) -> Result<(), TTSError>;
      fn pause(&mut self) -> Result<(), TTSError>;
      fn resume(&mut self) -> Result<(), TTSError>;
      fn stop(&mut self) -> Result<(), TTSError>;
      fn is_playing(&self) -> bool;
      fn is_paused(&self) -> bool;
      fn skip_forward(&mut self, seconds: f32);
      fn skip_backward(&mut self, seconds: f32);
      fn get_progress(&self) -> f32;  // 0.0 to 1.0
      fn get_frequency_bands(&self, num_bands: usize) -> Vec<f32>;
      fn name(&self) -> &str;
      fn validate_config(&self) -> bool;
  }
  ```
- [x] Define `TTSError` enum

### 1.2 Implement PiperTTSProvider
- [ ] Create `src/providers/piper.rs`
- [ ] Add dependencies: `rodio`, `symphonia`
- [ ] Implement Piper subprocess spawning
- [ ] Implement audio playback with rodio
- [ ] Implement pause/resume/stop
- [ ] Implement skip forward/backward
- [ ] Implement progress tracking
- [ ] Implement FFT for frequency bands (use `rustfft`)

### 1.3 Reorganize system module
- [x] Create `src/system/mod.rs`
- [x] Move clipboard code to `src/system/clipboard.rs`

---

## Phase 2: Async Application

### 2.1 Switch from Sandbox to Application
- [ ] Update `app.rs` to implement `Application` trait
- [ ] Add `Command` support for async operations
- [ ] Add `Subscription` for periodic updates

### 2.2 Add Subscriptions
- [ ] Progress bar updates (every ~33ms for smooth animation)
- [ ] Waveform animation (every ~75ms)
- [ ] Playback status polling (every ~100ms)

### 2.3 Integrate TTS with UI
- [ ] Store `TTSProvider` in App state (behind Arc<Mutex<>>)
- [ ] Connect play/pause/stop buttons to provider
- [ ] Connect skip buttons to provider
- [ ] Update progress bar from provider
- [ ] Update waveform from frequency bands

---

## Phase 3: Configuration

### 3.1 Create config module
- [ ] Create `src/config.rs`
- [ ] Define config structure:
  ```rust
  pub struct Config {
      pub voice_provider: String,  // "piper" or "polly"
      pub log_level: String,
  }
  ```
- [ ] Implement load/save to `~/.config/grars/config.json`
- [ ] Add serde dependency

### 3.2 Add settings menu
- [ ] Create settings popover in view
- [ ] Provider dropdown (Piper/Polly)
- [ ] Log level dropdown
- [ ] Persist changes on selection

---

## Phase 4: Polish

### 4.1 Error handling
- [ ] Display TTS errors in UI
- [ ] Graceful fallback if provider unavailable

### 4.2 Real waveform visualization
- [ ] Replace static bars with FFT-driven animation
- [ ] Smooth interpolation between updates

### 4.3 Auto-close on completion
- [ ] Close window when playback finishes
- [ ] Handle edge cases (stop, error)

---

## Phase 5: Future Enhancements

### 5.1 AWS Polly provider
- [ ] Create `src/providers/polly.rs`
- [ ] Add AWS SDK dependencies
- [ ] Implement streaming audio

### 5.2 CLI interface
- [ ] Add clap for argument parsing
- [ ] `grars speak-selection`
- [ ] `grars speak "text"`
- [ ] `grars --help`

---

## Dependencies to Add

```toml
[dependencies]
iced = { version = "0.12", features = ["svg"] }
rodio = "0.19"              # Audio playback
rustfft = "6.2"             # FFT for visualization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"          # Config persistence
thiserror = "2.0"           # Error handling
dirs = "5.0"                # Config directory paths
```

---

## Reference: original Python sources

- `src/python_app/providers/base.py` — TTSProvider interface
- `src/python_app/providers/piper.py` — Piper implementation with sounddevice
- `src/python_app/ui/window.py` — GTK4 UI, waveform animation, settings
- `src/python_app/utils/config.py` — JSON config management
- `src/python_app/utils/clipboard.py` — Clipboard/selection reading

