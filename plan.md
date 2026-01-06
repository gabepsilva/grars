# grars Implementation Plan

Rewriting the original Python GTK4 TTS app in Rust with Iced.

## Current State

✅ Fully functional Iced TTS application with:
- Floating borderless window
- Play/pause/stop buttons (fully functional)
- Real-time waveform visualization (FFT-driven)
- Dynamic progress bar (from TTS provider)
- Clipboard/selection reading at startup
- Piper and AWS Polly TTS providers
- Settings window with provider and log level selection
- Config persistence
- Error handling and display
- Auto-close on playback completion

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
│   └── polly.rs         # AWS Polly implementation
├── system/
│   ├── mod.rs
│   └── clipboard.rs     # Selection/clipboard reading
└── config.rs            # Config management
```

---

## Phase 1: Core TTS Infrastructure

### 1.1 Create TTSProvider trait
- [x] Create `src/providers/mod.rs`
- [x] Define `TTSProvider` trait (implemented with all core methods):
  ```rust
  pub trait TTSProvider {
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
  }
  ```
- [x] Define `TTSError` enum

### 1.2 Implement PiperTTSProvider
- [x] Create `src/providers/piper.rs`
- [x] Add dependencies: `rodio`, `rustfft`
- [x] Implement Piper subprocess spawning
- [x] Implement audio playback with rodio
- [x] Implement pause/resume/stop
- [x] Implement skip forward/backward
- [x] Implement progress tracking
- [x] Implement FFT for frequency bands (use `rustfft`)

### 1.3 Reorganize system module
- [x] Create `src/system/mod.rs`
- [x] Move clipboard code to `src/system/clipboard.rs`

---

## Phase 2: Async Application

### 2.1 Switch from Sandbox to Application
- [x] Update `app.rs` to implement `Application` trait
- [x] Add `Command` support for async operations
- [x] Add `Subscription` for periodic updates

### 2.2 Add Subscriptions
- [x] Progress bar updates (every ~75ms via Tick subscription)
- [x] Waveform animation (every ~75ms)
- [x] Playback status polling (every ~75ms)

### 2.3 Integrate TTS with UI
- [x] Store `TTSProvider` in App state (Option<Box<dyn TTSProvider>>)
- [x] Connect play/pause/stop buttons to provider
- [x] Connect skip buttons to provider
- [x] Update progress bar from provider
- [x] Update waveform from frequency bands

---

## Phase 3: Configuration

### 3.1 Create config module
- [x] Create `src/config.rs`
- [x] Define config structure (RawConfig with serde)
- [x] Implement load/save to `~/.config/grars/config.json`
- [x] Add serde dependency

### 3.2 Add settings menu
- [x] Create settings window in view
- [x] Provider radio buttons (Piper/Polly)
- [x] Log level radio buttons
- [x] Persist changes on selection

---

## Phase 4: Polish

### 4.1 Error handling
- [x] Display TTS errors in UI (error_message field, displayed in settings)
- [x] Graceful fallback if provider unavailable (error messages shown)

### 4.2 Real waveform visualization
- [x] Replace static bars with FFT-driven animation
- [x] Smooth interpolation between updates (via Tick subscription)

### 4.3 Auto-close on completion
- [x] Close window when playback finishes
- [x] Handle edge cases (stop, error)

---

## Phase 5: Future Enhancements

### 5.1 AWS Polly provider
- [x] Create `src/providers/polly.rs`
- [x] Add AWS SDK dependencies
- [x] Implement streaming audio (via AWS SDK)

### 5.2 CLI interface
- [ ] Add clap for argument parsing
- [ ] `grars speak-selection`
- [ ] `grars speak "text"`
- [ ] `grars --help`

---

## Dependencies (All Added ✅)

```toml
[dependencies]
iced = { version = "0.14", features = ["svg", "tokio"] }
rodio = "0.19"              # Audio playback
rustfft = "6.2"             # FFT for visualization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"          # Config persistence
thiserror = "2.0"           # Error handling
dirs = "5.0"                # Config directory paths
aws-config = "1.6"          # AWS SDK configuration
aws-sdk-polly = "1.76"      # AWS Polly TTS
tokio = { version = "1", features = ["rt-multi-thread"] }
tracing = "0.1"             # Structured logging
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
tracing-appender = "0.2"    # Log file appender
chrono = "0.4"              # Timestamp formatting
```

---

## Reference: original Python sources

- `src/python_app/providers/base.py` — TTSProvider interface
- `src/python_app/providers/piper.py` — Piper implementation with sounddevice
- `src/python_app/ui/window.py` — GTK4 UI, waveform animation, settings
- `src/python_app/utils/config.py` — JSON config management
- `src/python_app/utils/clipboard.py` — Clipboard/selection reading

