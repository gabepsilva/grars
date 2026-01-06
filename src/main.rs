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

use iced::{application, Size};
use tracing::{debug, info};

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

    // Read selected text at startup
    let selected_text = crate::system::get_selected_text();

    if let Some(ref text) = selected_text {
        info!(bytes = text.len(), "Text selected");
    } else {
        debug!("No text selected");
    }

    // Store selected text for later initialization after window appears
    crate::app::set_initial_text(selected_text);

    // Start the application immediately - window will appear right away
    application(crate::app::new, crate::app::update, crate::app::view)
        .title(crate::app::title)
        .subscription(crate::app::subscription)
        .window(iced::window::Settings {
            size: Size::new(360.0, 70.0),
            resizable: false,
            decorations: false,
            transparent: true,
            ..Default::default()
        })
        .run()
}
