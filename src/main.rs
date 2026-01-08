//! Entry point and window configuration

mod app;
mod config;
mod logging;
mod model;
mod providers;
mod styles;
mod system;
mod update;
mod view;

use iced::daemon;
use tracing::info;

fn main() -> iced::Result {
    // Initialize logging first (before anything else)
    let log_config = logging::LoggingConfig {
        verbosity: config::load_log_level(),
        log_to_stderr: true,
        log_to_file: true,
        log_dir: None, // Use default: ~/.local/share/grars/logs
    };

    if let Err(e) = logging::init_logging(&log_config) {
        eprintln!("Failed to initialize logging: {e}");
        // Continue anyway - app can run without logging
    }

    info!("grars starting up");

    // Use daemon for multi-window support (view receives window::Id)
    // Note: Text selection is now fetched asynchronously after UI appears for blazing fast startup
    daemon(crate::app::new, crate::app::update, crate::app::view)
        .title(crate::app::title)
        .subscription(crate::app::subscription)
        .run()
}
