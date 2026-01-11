//! Persistent configuration handling for Insight Reader.
//!
//! Persists the selected voice provider and log level in a simple JSON file:
//! `~/.config/insight-reader/config.json` with fields like:
//! `{ "voice_provider": "piper", "log_level": "INFO" }`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use dirs::config_dir;
use tracing::{debug, error, warn};

use crate::model::{LogLevel, OCRBackend, TTSBackend};

const APP_CONFIG_DIR_NAME: &str = "insight-reader";
const CONFIG_FILE_NAME: &str = "config.json";

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Json(serde_json::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "IO error: {err}"),
            Self::Json(err) => write!(f, "JSON error: {err}"),
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
struct RawConfig {
    /// Voice provider name ("piper" or "polly").
    #[serde(default)]
    voice_provider: Option<String>,

    /// Log level string, kept for compatibility (unused for now).
    #[serde(default)]
    log_level: Option<String>,

    /// Whether text cleanup is enabled (sends text to cleanup API before TTS).
    #[serde(default)]
    text_cleanup_enabled: Option<bool>,

    /// Selected Piper voice key (e.g., "en_US-lessac-medium").
    #[serde(default)]
    selected_voice: Option<String>,
    /// Selected AWS Polly voice ID (e.g., "Matthew", "Joanna").
    #[serde(default)]
    selected_polly_voice: Option<String>,

    /// OCR backend name ("default" or "better_ocr").
    #[serde(default)]
    ocr_backend: Option<String>,
}

fn config_path() -> Option<PathBuf> {
    let path = config_dir()?.join(APP_CONFIG_DIR_NAME).join(CONFIG_FILE_NAME);
    Some(path)
}

fn ensure_config_dir_exists(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn load_raw_config() -> Result<RawConfig, ConfigError> {
    let Some(path) = config_path() else {
        // No config directory available on this platform; treat as empty config.
        debug!("No config_dir available, using defaults only");
        return Ok(RawConfig::default());
    };

    if !path.exists() {
        debug!(?path, "Config file does not exist, using defaults");
        return Ok(RawConfig::default());
    }

    let data = fs::read_to_string(&path)?;
    let cfg = serde_json::from_str(&data)?;
    debug!(?path, "Config loaded");
    Ok(cfg)
}

fn save_raw_config(mut cfg: RawConfig) -> Result<(), ConfigError> {
    let Some(path) = config_path() else {
        // Nothing we can do; silently ignore.
        warn!("No config_dir available, skipping save");
        return Ok(());
    };

    ensure_config_dir_exists(&path)?;
    // Normalize by dropping empty strings if present.
    cfg.selected_polly_voice = cfg.selected_polly_voice.filter(|s| !s.is_empty());
    cfg.voice_provider = cfg.voice_provider.filter(|s| !s.is_empty());
    cfg.log_level = cfg.log_level.filter(|s| !s.is_empty());
    cfg.selected_voice = cfg.selected_voice.filter(|s| !s.is_empty());
    cfg.ocr_backend = cfg.ocr_backend.filter(|s| !s.is_empty());

    let data = serde_json::to_string_pretty(&cfg)?;
    fs::write(&path, data)?;
    debug!(?path, "Config saved");
    Ok(())
}

fn backend_from_str(s: &str) -> Option<TTSBackend> {
    match s {
        "piper" => Some(TTSBackend::Piper),
        "polly" => Some(TTSBackend::AwsPolly),
        _ => None,
    }
}

fn backend_to_str(backend: TTSBackend) -> &'static str {
    match backend {
        TTSBackend::Piper => "piper",
        TTSBackend::AwsPolly => "polly",
    }
}

fn log_level_from_str(s: &str) -> Option<LogLevel> {
    match s.to_ascii_uppercase().as_str() {
        "ERROR" => Some(LogLevel::Error),
        "WARN" | "WARNING" => Some(LogLevel::Warn),
        "INFO" => Some(LogLevel::Info),
        "DEBUG" => Some(LogLevel::Debug),
        "TRACE" => Some(LogLevel::Trace),
        _ => None,
    }
}

fn log_level_to_str(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Error => "ERROR",
        LogLevel::Warn => "WARN",
        LogLevel::Info => "INFO",
        LogLevel::Debug => "DEBUG",
        LogLevel::Trace => "TRACE",
    }
}

/// Load the persisted voice provider, defaulting to Piper if not set or invalid.
pub fn load_voice_provider() -> TTSBackend {
    let backend = match load_raw_config() {
        Ok(cfg) => cfg
            .voice_provider
            .as_deref()
            .and_then(backend_from_str)
            .unwrap_or(TTSBackend::Piper),
        Err(err) => {
            warn!(error = ?err, "Failed to load config, using default backend");
            TTSBackend::Piper
        }
    };
    debug!(?backend, "Loaded voice provider");
    backend
}

/// Load the persisted log level, defaulting to `Info` if not set or invalid.
pub fn load_log_level() -> LogLevel {
    match load_raw_config() {
        Ok(cfg) => cfg
            .log_level
            .as_deref()
            .and_then(log_level_from_str)
            .unwrap_or(LogLevel::Info),
        Err(err) => {
            // Note: we can't use tracing here as logging may not be initialized yet
            eprintln!("Config: failed to load config, using default log level: {err:?}");
            LogLevel::Info
        }
    }
}

/// Load config or return default on error.
fn load_or_default_config() -> RawConfig {
    match load_raw_config() {
        Ok(cfg) => cfg,
        Err(err) => {
            warn!(error = ?err, "Failed to load existing config, starting fresh");
            RawConfig::default()
        }
    }
}

/// Persist the selected voice provider to disk.
///
/// Errors are logged and otherwise ignored.
pub fn save_voice_provider(backend: TTSBackend) {
    debug!(?backend, "Saving voice provider");
    let mut cfg = load_or_default_config();
    cfg.voice_provider = Some(backend_to_str(backend).to_string());
    if let Err(err) = save_raw_config(cfg) {
        error!(error = ?err, "Failed to save config");
    }
}

/// Persist the selected log level to disk.
///
/// Errors are logged and otherwise ignored.
pub fn save_log_level(level: LogLevel) {
    debug!(?level, "Saving log level");
    let mut cfg = load_or_default_config();
    cfg.log_level = Some(log_level_to_str(level).to_string());
    if let Err(err) = save_raw_config(cfg) {
        error!(error = ?err, "Failed to save config");
    }
}

/// Load the persisted text cleanup enabled setting, defaulting to `false` if not set.
pub fn load_text_cleanup_enabled() -> bool {
    match load_raw_config() {
        Ok(cfg) => cfg.text_cleanup_enabled.unwrap_or(false),
        Err(err) => {
            warn!(error = ?err, "Failed to load config, text cleanup disabled by default");
            false
        }
    }
}

/// Persist the text cleanup enabled setting to disk.
///
/// Errors are logged and otherwise ignored.
pub fn save_text_cleanup_enabled(enabled: bool) {
    debug!(?enabled, "Saving text cleanup enabled");
    let mut cfg = load_or_default_config();
    cfg.text_cleanup_enabled = Some(enabled);
    if let Err(err) = save_raw_config(cfg) {
        error!(error = ?err, "Failed to save config");
    }
}

/// Load the persisted selected voice, returning None if not set or invalid.
pub fn load_selected_voice() -> Option<String> {
    match load_raw_config() {
        Ok(cfg) => cfg.selected_voice.filter(|s| !s.is_empty()),
        Err(err) => {
            warn!(error = ?err, "Failed to load config, no voice selected");
            None
        }
    }
}

/// Persist the selected voice to disk.
///
/// Errors are logged and otherwise ignored.
pub fn save_selected_voice(voice_key: String) {
    debug!(voice_key = %voice_key, "Saving selected voice");
    let mut cfg = load_or_default_config();
    cfg.selected_voice = Some(voice_key);
    if let Err(err) = save_raw_config(cfg) {
        error!(error = ?err, "Failed to save config");
    }
}

/// Load the persisted selected AWS Polly voice, returning None if not set or invalid.
pub fn load_selected_polly_voice() -> Option<String> {
    match load_raw_config() {
        Ok(cfg) => cfg.selected_polly_voice.filter(|s| !s.is_empty()),
        Err(err) => {
            warn!(error = ?err, "Failed to load config, no AWS voice selected");
            None
        }
    }
}

/// Persist the selected AWS Polly voice to disk.
///
/// Errors are logged and otherwise ignored.
pub fn save_selected_polly_voice(voice_id: String) {
    debug!(voice_id = %voice_id, "Saving selected AWS Polly voice");
    let mut cfg = load_or_default_config();
    cfg.selected_polly_voice = Some(voice_id);
    if let Err(err) = save_raw_config(cfg) {
        error!(error = ?err, "Failed to save config");
    }
}

fn ocr_backend_from_str(s: &str) -> Option<OCRBackend> {
    match s {
        "default" => Some(OCRBackend::Default),
        "better_ocr" => Some(OCRBackend::BetterOCR),
        _ => None,
    }
}

fn ocr_backend_to_str(backend: OCRBackend) -> &'static str {
    match backend {
        OCRBackend::Default => "default",
        OCRBackend::BetterOCR => "better_ocr",
    }
}

/// Load the persisted OCR backend, defaulting to `Default` if not set.
pub fn load_ocr_backend() -> OCRBackend {
    match load_raw_config() {
        Ok(cfg) => {
            cfg.ocr_backend
                .and_then(|s| ocr_backend_from_str(&s))
                .unwrap_or(OCRBackend::Default)
        }
        Err(err) => {
            warn!(error = ?err, "Failed to load config, using default OCR backend");
            OCRBackend::Default
        }
    }
}

/// Persist the OCR backend to disk.
///
/// Errors are logged and otherwise ignored.
pub fn save_ocr_backend(backend: OCRBackend) {
    debug!(?backend, "Saving OCR backend");
    let mut cfg = load_or_default_config();
    cfg.ocr_backend = Some(ocr_backend_to_str(backend).to_string());
    if let Err(err) = save_raw_config(cfg) {
        error!(error = ?err, "Failed to save config");
    }
}
