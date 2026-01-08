//! Business logic for state transitions

use iced::window;
use iced::{Size, Task};
use std::sync::mpsc;
use tracing::{debug, error, info, trace, warn};

use crate::config;
use crate::logging;
use crate::model::{App, Message, PlaybackState, TTSBackend};
use crate::providers::{PiperTTSProvider, PollyTTSProvider, TTSProvider};

// Wrapper to make TTSProvider Send (required for cross-thread usage)
// SAFETY: This is safe because we only move the provider between threads during initialization,
// and rodio's types are actually safe to send between threads in practice.
struct SendTTSProvider(Box<dyn TTSProvider>);
unsafe impl Send for SendTTSProvider {}

// Static storage for provider during async initialization
static PENDING_PROVIDER: std::sync::Mutex<Option<SendTTSProvider>> = std::sync::Mutex::new(None);

const SKIP_SECONDS: f32 = 5.0;
const NUM_BANDS: usize = 10;

/// Check if an error string indicates an AWS credential/authentication issue.
fn is_aws_credential_error(error_str: &str) -> bool {
    error_str.contains("credentials")
        || error_str.contains("authentication")
        || error_str.contains("Unauthorized")
        || error_str.contains("dispatch failure")
        || error_str.contains("AWS")
}

/// Format TTS error message, handling AWS credential errors specially.
fn format_tts_error(error: &str, backend: TTSBackend) -> String {
    if backend == TTSBackend::AwsPolly && is_aws_credential_error(error) {
        PollyTTSProvider::check_credentials()
            .err()
            .unwrap_or_else(|| error.to_string())
    } else {
        error.to_string()
    }
}

/// Handle skip forward/backward operations with shared logic.
fn handle_skip<F>(app: &mut App, skip_fn: F, direction: &str) -> Task<Message>
where
    F: FnOnce(&mut dyn TTSProvider),
{
    if let Some(ref mut provider) = app.provider {
        trace!(seconds = SKIP_SECONDS, direction, "Skip requested");
        skip_fn(provider.as_mut());
        app.progress = provider.get_progress();
        debug!(progress = app.progress, direction, "Skip applied");
    } else {
        warn!(direction, "Skip received with no active provider");
    }
    Task::none()
}

/// Open the settings window with error display enabled.
/// Returns the window ID and task mapped to Message::WindowOpened.
fn open_settings_window() -> (window::Id, Task<Message>) {
    let (window_id, task) = window::open(window::Settings {
        size: Size::new(760.0, 360.0),
        resizable: false,
        decorations: false,
        transparent: false,
        visible: true,
        position: window::Position::Centered,
        ..Default::default()
    });
    (window_id, task.map(Message::WindowOpened))
}

/// Initialize TTS provider and start speaking with the given text asynchronously.
/// Returns a Task that will complete when synthesis is done.
/// This prevents blocking the UI thread during TTS synthesis.
fn initialize_tts_async(
    backend: TTSBackend,
    text: String,
    context: &'static str,
) -> Task<Message> {
    info!(
        context,
        backend = ?backend,
        bytes = text.len(),
        "Starting async TTS initialization"
    );

    // Check AWS credentials before attempting to initialize (synchronous, fast)
    if backend == TTSBackend::AwsPolly {
        if let Err(e) = PollyTTSProvider::check_credentials() {
            warn!("AWS credentials not found during initialization");
            return Task::perform(
                async move { Err(e) },
                Message::TTSInitialized,
            );
        }
    }

    // Create provider (this is fast and happens on main thread)
    let provider_result = match backend {
        TTSBackend::Piper => PiperTTSProvider::new().map(|p| Box::new(p) as Box<dyn TTSProvider>),
        TTSBackend::AwsPolly => PollyTTSProvider::new().map(|p| Box::new(p) as Box<dyn TTSProvider>),
    }
    .map_err(|e| format!("{}", e));

    match provider_result {
        Ok(provider) => {
            // Wrap provider to make it Send-safe for cross-thread usage
            let send_provider = SendTTSProvider(provider);
            
            // Spawn thread to do blocking synthesis
            let (tx, rx) = mpsc::channel();
            
            std::thread::spawn(move || {
                let mut send_provider = send_provider;
                let provider = &mut send_provider.0;
                let result = provider.speak(&text);
                
                match result {
                    Ok(()) => {
                        info!(context, "TTS synthesis completed successfully");
                        if let Ok(mut guard) = PENDING_PROVIDER.lock() {
                            *guard = Some(send_provider);
                        }
                        let _ = tx.send(Ok(()));
                    }
                    Err(e) => {
                        error!(error = %e, "TTS speak failed");
                        let error_msg = format_tts_error(&format!("{}", e), backend);
                        let _ = tx.send(Err(error_msg));
                    }
                }
            });

            // Return a task that waits for synthesis (non-blocking for UI)
            Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        rx.recv().unwrap_or_else(|e| Err(format!("Channel error: {}", e)))
                    })
                    .await
                    .unwrap_or_else(|e| Err(format!("Task join error: {}", e)))
                },
                Message::TTSInitialized,
            )
        }
        Err(e) => {
            let error_msg = format_tts_error(&e, backend);
            Task::perform(
                async move { Err(error_msg) },
                Message::TTSInitialized,
            )
        }
    }
}

pub fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::SkipBackward => {
            handle_skip(app, |p| p.skip_backward(SKIP_SECONDS), "backward")
        }
        Message::SkipForward => {
            handle_skip(app, |p| p.skip_forward(SKIP_SECONDS), "forward")
        }
        Message::PlayPause => {
            if let Some(ref mut provider) = app.provider {
                match app.playback_state {
                    PlaybackState::Playing => {
                        match provider.pause() {
                            Ok(()) => {
                                app.playback_state = PlaybackState::Paused;
                                info!("Playback paused");
                            }
                            Err(e) => {
                                error!(error = %e, "Failed to pause playback");
                            }
                        };
                    }
                    PlaybackState::Paused => {
                        match provider.resume() {
                            Ok(()) => {
                                app.playback_state = PlaybackState::Playing;
                                info!("Playback resumed");
                            }
                            Err(e) => {
                                error!(error = %e, "Failed to resume playback");
                            }
                        };
                    }
                    PlaybackState::Stopped => {}
                }
            } else {
                warn!("PlayPause received with no active provider");
            }
            Task::none()
        }
        Message::Stop => {
            if let Some(ref mut provider) = app.provider {
                if let Err(e) = provider.stop() {
                    error!(error = %e, "Failed to stop playback");
                }
            }
            app.playback_state = PlaybackState::Stopped;
            app.progress = 0.0;
            app.frequency_bands = vec![0.0; NUM_BANDS];
            app.is_loading = false;
            app.loading_animation_time = 0.0;
            info!("Playback stopped, closing main window");
            window::latest().and_then(window::close)
        }
        Message::Tick => {
            // Handle loading animation
            if app.is_loading {
                app.loading_animation_time += 0.15; // Increment animation time (faster animation)
                if app.loading_animation_time > std::f32::consts::PI * 2.0 {
                    app.loading_animation_time -= std::f32::consts::PI * 2.0;
                }
                
                // Generate animated bar values using sine waves
                // Creates a smooth wave that travels across the bars
                app.frequency_bands = (0..NUM_BANDS)
                    .map(|i| {
                        // Create a traveling wave effect
                        let position = i as f32 / NUM_BANDS as f32;
                        let wave = (app.loading_animation_time * 2.0 + position * std::f32::consts::PI * 2.0).sin();
                        // Add some variation with a secondary wave
                        let secondary = (app.loading_animation_time * 1.5 + position * std::f32::consts::PI * 3.0).sin() * 0.3;
                        // Normalize to 0.0-1.0 range with some minimum height
                        ((wave + secondary) * 0.4 + 0.5).clamp(0.2, 1.0)
                    })
                    .collect();
            } else if let Some(ref provider) = app.provider {
                app.progress = provider.get_progress();
                app.frequency_bands = provider.get_frequency_bands(NUM_BANDS);

                if !provider.is_playing() && !provider.is_paused() {
                    info!("Playback finished, stopping and closing window");
                    app.playback_state = PlaybackState::Stopped;
                    return window::latest().and_then(window::close);
                }
            } else {
                trace!("Tick received with no active provider");
            }
            Task::none()
        }
        Message::Settings => {
            // Prevent opening multiple settings windows
            if app.settings_window_id.is_some() {
                debug!("Settings window already open, ignoring request");
                return Task::none();
            }
            
            debug!("Settings clicked");
            let (window_id, task) = window::open(window::Settings {
                size: Size::new(760.0, 280.0),
                resizable: false,
                decorations: false,
                transparent: false,
                visible: true,
                position: window::Position::Centered,
                ..Default::default()
            });
            debug!(?window_id, "Opening settings window");
            app.settings_window_id = Some(window_id);
            app.show_settings_modal = true;
            task.map(Message::WindowOpened)
        }
        Message::CloseSettings => {
            app.show_settings_modal = false;
            if let Some(window_id) = app.settings_window_id.take() {
                window::close(window_id)
            } else {
                Task::none()
            }
        }
        Message::ProviderSelected(backend) => {
            info!(?backend, "TTS provider selected");
            app.selected_backend = backend;
            
            // Check AWS credentials if AWS Polly is selected
            if backend == TTSBackend::AwsPolly {
                match PollyTTSProvider::check_credentials() {
                    Ok(()) => {
                        app.error_message = None;
                        info!("AWS credentials found");
                    }
                    Err(e) => {
                        app.error_message = Some(e);
                        warn!("AWS credentials not found when selecting AWS Polly");
                    }
                }
            } else {
                // Clear error message when switching to Piper
                app.error_message = None;
            }
            
            // Persist the selected backend so future runs remember the choice.
            config::save_voice_provider(backend);
            Task::none()
        }
        Message::LogLevelSelected(level) => {
            info!(?level, "Log level selected");
            app.log_level = level;
            // Persist the selected log level so future runs remember the choice.
            config::save_log_level(level);
            // Update runtime log level
            logging::set_verbosity(level);
            Task::none()
        }
        Message::WindowOpened(id) => {
            info!(?id, "Window opened event received");
            if app.main_window_id.is_none() {
                app.main_window_id = Some(id);
                info!("Main window ID set - UI is now visible");
                
                // If we already have pending text (from async fetch), initialize TTS now
                if let Some(text) = app.pending_text.take() {
                    info!("Window opened with pending text, initializing TTS");
                    app.is_loading = true;
                    app.loading_animation_time = 0.0;
                    return initialize_tts_async(app.selected_backend, text, "WindowOpened");
                }
            } else {
                debug!(?id, "Window opened but main window ID already set");
            }
            app.current_window_id = Some(id);
            Task::none()
        }
        Message::WindowClosed(id) => {
            debug!(?id, "Window closed");
            if app.settings_window_id == Some(id) {
                app.settings_window_id = None;
                app.show_settings_modal = false;
            }
            if app.current_window_id == Some(id) {
                app.current_window_id = None;
            }
            // Exit when the main window is closed
            if app.main_window_id == Some(id) {
                info!("Main window closed, exiting");
                return iced::exit();
            }
            Task::none()
        }
        Message::SelectedTextFetched(text) => {
            info!("Selected text fetched asynchronously");
            if let Some(ref text) = text {
                info!(bytes = text.len(), preview = %text.chars().take(50).collect::<String>(), "Text selected");
            } else {
                info!("No text selected - app will wait for text or close");
            }
            
                // Initialize TTS if window is already open, otherwise store for later
            if app.main_window_id.is_some() {
                if let Some(text) = text {
                    info!("Window ready, initializing TTS with fetched text");
                    app.is_loading = true;
                    app.loading_animation_time = 0.0;
                    return initialize_tts_async(app.selected_backend, text, "SelectedTextFetched");
                } else {
                    warn!("No text selected - closing window");
                    return window::latest().and_then(window::close);
                }
            } else {
                // Window not ready yet, store text for WindowOpened handler
                app.pending_text = text;
                trace!("Window not ready yet, text stored for later initialization");
            }
            Task::none()
        }
        Message::TTSInitialized(result) => {
            // Clear loading state regardless of result
            app.is_loading = false;
            app.loading_animation_time = 0.0;
            
            match result {
                Ok(()) => {
                    // Retrieve provider from static storage
                    if let Ok(mut guard) = PENDING_PROVIDER.lock() {
                        if let Some(send_provider) = guard.take() {
                            app.provider = Some(send_provider.0);
                            app.playback_state = PlaybackState::Playing;
                            app.error_message = None;
                            info!("TTS provider initialized and playback started");
                        } else {
                            error!("TTS initialization succeeded but no provider found in storage");
                            app.error_message = Some("Internal error: provider not found".to_string());
                        }
                    } else {
                        error!("Failed to lock PENDING_PROVIDER mutex");
                        app.error_message = Some("Internal error: mutex lock failed".to_string());
                    }
                }
                Err(e) => {
                    error!(error = %e, "TTS initialization failed");
                    app.error_message = Some(e);
                    // Auto-open settings if there's an error and settings aren't already open
                    if app.settings_window_id.is_none() {
                        let (window_id, task) = open_settings_window();
                        app.settings_window_id = Some(window_id);
                        app.show_settings_modal = true;
                        return task;
                    }
                }
            }
            Task::none()
        }
    }
}

