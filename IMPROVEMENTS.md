# Code Improvements Checklist

## 🔴 High Priority

### Code Organization
- [ ] Split `view.rs` (1616 lines) into submodules:
  - [ ] `view/main.rs` - main view
  - [ ] `view/settings.rs` - settings window
  - [ ] `view/voice_selection.rs` - voice selection window
  - [ ] `view/screenshot.rs` - screenshot viewer
  - [ ] `view/dialogs.rs` - info dialogs
- [ ] Split `update.rs` (924 lines) by domain:
  - [ ] `update/tts.rs` - TTS-related message handling
  - [ ] `update/windows.rs` - window management
  - [ ] `update/voices.rs` - voice selection/download
  - [ ] `update/screenshot.rs` - screenshot handling
  - [ ] `update/main.rs` - main update function

### Replace Shell Commands with Libraries
- [x] Replace clipboard operations with `arboard` crate:
  - [x] Remove `pbcopy`/`pbpaste` shell commands (macOS) ✅
  - [x] Remove `xclip`/`wl-copy` shell commands (Linux) ✅
  - [x] Update `src/system/clipboard.rs` to use `arboard` (macOS and Linux) ✅
- [x] Replace URL opening with `open` crate:
  - [x] Remove `Command::new("open")` (macOS) ✅
  - [x] Remove `Command::new("xdg-open")` (Linux) ✅
  - [x] Remove `Command::new("cmd /c start")` (Windows) ✅
  - [x] Update `open_url()` function in `src/update.rs` ✅

## 🟡 Medium Priority

### Safety & Architecture
- [ ] Fix unsafe `Send` implementation:
  - [ ] Remove `unsafe impl Send for SendTTSProvider`
  - [ ] Add `Send` bound to `TTSProvider` trait, OR
  - [ ] Use `Arc<Mutex<dyn TTSProvider>>` instead
- [ ] Remove static mutex pattern:
  - [ ] Remove `static PENDING_PROVIDER` mutex
  - [ ] Pass provider through async chain or use channels
  - [ ] Update `initialize_tts_async()` function
- [ ] Simplify async patterns:
  - [ ] Remove mixing of `std::thread::spawn` + `mpsc::channel` + `tokio::task::spawn_blocking`
  - [ ] Use Tokio's blocking thread pool consistently
  - [ ] Or use channels throughout (no thread spawning)

### Code Abstraction
- [ ] Create `WindowManager` struct:
  - [ ] Encapsulate window opening/closing logic
  - [ ] Reduce repetitive window management code
  - [ ] Centralize window ID tracking
- [ ] Extract window management helpers:
  - [ ] `open_settings_window()` → `WindowManager::open_settings()`
  - [ ] `open_info_window()` → `WindowManager::open_info()`
  - [ ] `close_window_if_some()` → `WindowManager::close()`

### Error Handling
- [ ] Replace string matching with structured errors:
  - [ ] Remove `is_aws_credential_error()` string matching
  - [ ] Use AWS SDK's structured error types
  - [ ] Update `format_tts_error()` to use error types

## 🟢 Low Priority

### Libraries & Dependencies
- [ ] Use library for flag emojis:
  - [ ] Replace manual `get_flag_emoji()` mapping (100+ lines)
  - [ ] Consider `country-emoji` or similar crate
  - [ ] Or use a smaller data structure

### Configuration & Constants
- [ ] Extract magic numbers to constants:
  - [ ] `75ms` tick interval → `const TICK_INTERVAL_MS: u64 = 75`
  - [ ] Window dimensions → `const WINDOW_*` constants
  - [ ] Audio visualization values → `const AUDIO_*` constants
- [ ] Create config module for hardcoded values:
  - [ ] URLs (API endpoints, voice repository)
  - [ ] File paths and directory names
  - [ ] Default values

### Testing
- [ ] Add unit tests for pure functions:
  - [ ] `config.rs` helper functions (`backend_from_str`, `log_level_from_str`, etc.)
  - [ ] `voices/mod.rs` (`parse_voices_json`, `get_available_languages`)
  - [ ] `update.rs` (`is_aws_credential_error`, `format_tts_error`)
  - [ ] `view.rs` (`engine_display_name`, `get_flag_emoji`, `bar_height`)
  - [x] `system/clipboard.rs` (`text_preview`) ✅ - Added comprehensive tests (25 tests covering all clipboard functionality)

### Code Quality
- [ ] Reduce code duplication:
  - [x] Extract common clipboard try/fallback pattern ✅ - macOS now uses arboard (no fallback needed), Linux still has fallback pattern
  - [ ] Extract common error message formatting
  - [ ] Extract common window opening patterns
- [ ] Improve type safety:
  - [ ] Use newtype patterns for window IDs
  - [ ] Use enums instead of string matching where possible
- [ ] Add documentation:
  - [ ] Document complex async patterns
  - [ ] Add examples for public APIs
  - [ ] Document unsafe blocks with safety invariants

## 📝 Notes

### Current Issues Summary
- **Unsafe usage**: 1 instance (questionable `Send` implementation)
- **Static state**: Global mutex for provider storage
- **Large files**: `view.rs` (1616 lines), `update.rs` (924 lines)
- **Shell dependencies**: macOS clipboard now uses `arboard` crate (no shell commands). Linux clipboard and URL opening still rely on external commands
- **String matching**: Fragile error detection via string contains

### Estimated Impact
- **High priority**: Improves maintainability, reduces dependencies, fixes safety issues
- **Medium priority**: Improves architecture, reduces complexity
- **Low priority**: Nice-to-have improvements, better DX
