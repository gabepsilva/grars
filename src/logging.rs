//! Centralized logging infrastructure with human-friendly formatting.
//!
//! Provides a `tracing`-based logging system with:
//! - Human-readable format: timestamp, level, target, file:line, message
//! - Runtime log level control via GUI and `RUST_LOG` environment variable
//! - Dual output to stderr and rotating log files

use std::fmt;
use std::io;
use std::path::PathBuf;
use std::sync::OnceLock;

use tracing::{Event, Level, Subscriber};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::format::{self, FormatEvent, FormatFields};
use tracing_subscriber::fmt::FmtContext;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{reload, EnvFilter};

use crate::model::LogLevel;

/// Global handle to reload the log filter at runtime.
static FILTER_HANDLE: OnceLock<reload::Handle<EnvFilter, tracing_subscriber::Registry>> =
    OnceLock::new();

/// Error type for logging initialization failures.
#[derive(Debug)]
pub enum LogInitError {
    /// Failed to create log directory
    DirectoryCreation(io::Error),
    /// Failed to set global subscriber (already initialized)
    AlreadyInitialized,
    /// Failed to parse log filter
    FilterParse(String),
}

impl std::fmt::Display for LogInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DirectoryCreation(e) => write!(f, "Failed to create log directory: {e}"),
            Self::AlreadyInitialized => write!(f, "Logging already initialized"),
            Self::FilterParse(s) => write!(f, "Failed to parse log filter: {s}"),
        }
    }
}

impl std::error::Error for LogInitError {}

/// Configuration for the logging system.
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Initial log verbosity level
    pub verbosity: LogLevel,
    /// Whether to log to stderr
    pub log_to_stderr: bool,
    /// Whether to log to a file
    pub log_to_file: bool,
    /// Directory for log files (None = use default app data dir)
    pub log_dir: Option<PathBuf>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            verbosity: LogLevel::Info,
            log_to_stderr: true,
            log_to_file: true,
            log_dir: None,
        }
    }
}

/// Convert `LogLevel` to an `EnvFilter` directive string.
fn log_level_to_filter_string(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Error => "error",
        LogLevel::Warn => "warn",
        LogLevel::Info => "info",
        LogLevel::Debug => "debug",
        LogLevel::Trace => "trace",
    }
}

/// Build an `EnvFilter` from `RUST_LOG` or fall back to the configured level.
fn build_env_filter(default_level: LogLevel) -> Result<EnvFilter, LogInitError> {
    // Check RUST_LOG first
    if let Ok(rust_log) = std::env::var("RUST_LOG") {
        if !rust_log.is_empty() {
            return EnvFilter::try_new(&rust_log)
                .map_err(|e| LogInitError::FilterParse(e.to_string()));
        }
    }

    // Fall back to configured level
    let filter_str = log_level_to_filter_string(default_level);
    EnvFilter::try_new(filter_str).map_err(|e| LogInitError::FilterParse(e.to_string()))
}

/// Resolve the log directory path.
fn resolve_log_dir(config: &LoggingConfig) -> PathBuf {
    config.log_dir.clone().unwrap_or_else(|| {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("grars")
            .join("logs")
    })
}

/// Human-friendly log formatter.
///
/// Produces lines like:
/// ```text
/// 2026-01-06T15:23:41.512Z INFO grars::view (view.rs:92) – Rendered main window
/// ```
struct HumanFormatter;

impl<S, N> FormatEvent<S, N> for HumanFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        // 1. Timestamp
        let now = chrono::Utc::now();
        write!(writer, "{} ", now.format("%Y-%m-%dT%H:%M:%S%.3fZ"))?;

        // 2. Level (colored for terminals)
        let level = *event.metadata().level();
        let level_str = match level {
            Level::ERROR => "ERROR",
            Level::WARN => "WARN ",
            Level::INFO => "INFO ",
            Level::DEBUG => "DEBUG",
            Level::TRACE => "TRACE",
        };
        write!(writer, "{level_str} ")?;

        // 3. Target (module path)
        let target = event.metadata().target();
        write!(writer, "{target} ")?;

        // 4. File and line (if available)
        let file = event.metadata().file();
        let line = event.metadata().line();
        if let (Some(file), Some(line)) = (file, line) {
            // Extract just the filename from the path
            let filename = std::path::Path::new(file)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(file);
            write!(writer, "({filename}:{line}) ")?;
        }

        // 5. Separator
        write!(writer, "– ")?;

        // 6. Event fields/message
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

/// Initialize the logging system with the given configuration.
///
/// This should be called once at application startup, before any logging occurs.
/// After initialization, use `set_verbosity()` to change the log level at runtime.
///
/// # Errors
///
/// Returns an error if:
/// - The log directory cannot be created (when file logging is enabled)
/// - The subscriber has already been set (double initialization)
/// - The log filter string is invalid
pub fn init_logging(config: &LoggingConfig) -> Result<(), LogInitError> {
    // Build the initial filter
    let filter = build_env_filter(config.verbosity)?;

    // Wrap the filter in a reloadable layer
    let (filter_layer, filter_handle) = reload::Layer::new(filter);

    // Store the handle for runtime updates
    if FILTER_HANDLE.set(filter_handle).is_err() {
        return Err(LogInitError::AlreadyInitialized);
    }

    // Build stderr layer
    let stderr_layer = tracing_subscriber::fmt::layer()
        .event_format(HumanFormatter)
        .with_writer(io::stderr);

    // Handle the four configuration cases explicitly to satisfy the type system
    match (config.log_to_stderr, config.log_to_file) {
        (true, true) => {
            let log_dir = resolve_log_dir(config);
            std::fs::create_dir_all(&log_dir).map_err(LogInitError::DirectoryCreation)?;
            let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "grars.log");
            let file_layer = tracing_subscriber::fmt::layer()
                .event_format(HumanFormatter)
                .with_writer(file_appender)
                .with_ansi(false);

            tracing_subscriber::registry()
                .with(filter_layer)
                .with(stderr_layer)
                .with(file_layer)
                .init();
        }
        (true, false) => {
            tracing_subscriber::registry()
                .with(filter_layer)
                .with(stderr_layer)
                .init();
        }
        (false, true) => {
            let log_dir = resolve_log_dir(config);
            std::fs::create_dir_all(&log_dir).map_err(LogInitError::DirectoryCreation)?;
            let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "grars.log");
            let file_layer = tracing_subscriber::fmt::layer()
                .event_format(HumanFormatter)
                .with_writer(file_appender)
                .with_ansi(false);

            tracing_subscriber::registry()
                .with(filter_layer)
                .with(file_layer)
                .init();
        }
        (false, false) => {
            // No output configured - just set up the filter (unusual but valid)
            tracing_subscriber::registry().with(filter_layer).init();
        }
    }

    // Log that we've started
    tracing::info!(
        verbosity = ?config.verbosity,
        stderr = config.log_to_stderr,
        file = config.log_to_file,
        "Logging initialized"
    );

    Ok(())
}

/// Change the log verbosity level at runtime.
///
/// This is typically called when the user changes the log level in the GUI.
/// Has no effect if logging hasn't been initialized yet.
pub fn set_verbosity(level: LogLevel) {
    let Some(handle) = FILTER_HANDLE.get() else {
        // Logging not initialized; silently ignore
        eprintln!("Warning: set_verbosity called before logging initialized");
        return;
    };

    let filter_str = log_level_to_filter_string(level);
    match EnvFilter::try_new(filter_str) {
        Ok(new_filter) => {
            if let Err(e) = handle.reload(new_filter) {
                eprintln!("Failed to reload log filter: {e}");
            } else {
                tracing::info!(level = ?level, "Log level changed");
            }
        }
        Err(e) => {
            eprintln!("Failed to parse log filter '{filter_str}': {e}");
        }
    }
}

/// Get the default log directory path.
///
/// Useful for displaying to the user where logs are stored.
#[allow(dead_code)]
pub fn default_log_dir() -> PathBuf {
    resolve_log_dir(&LoggingConfig::default())
}

