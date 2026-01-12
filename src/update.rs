//! Business logic for state transitions

use iced::window;
use iced::{Size, Task};
use std::sync::mpsc;
use tracing::{debug, error, info, trace, warn};

use crate::config;
use crate::logging;
use crate::model::{App, Message, OCRBackend, PlaybackState, TTSBackend};
use crate::providers::{PiperTTSProvider, PollyTTSProvider, TTSProvider};
use crate::system;

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

/// Set loading state on the app with a status message.
fn set_loading_state(app: &mut App, status: &str) {
    app.is_loading = true;
    app.loading_animation_time = 0.0;
    app.status_text = Some(status.to_string());
}

/// Clear loading state on the app.
fn clear_loading_state(app: &mut App) {
    app.is_loading = false;
    app.loading_animation_time = 0.0;
    app.status_text = None;
}

/// Open a URL in the default browser (cross-platform).
fn open_url(url: &str) {
    if let Err(e) = open::that(url) {
        error!("Failed to open URL '{}': {}", url, e);
    }
}

/// Open the settings window with error display enabled.
/// Returns the window ID and task mapped to Message::WindowOpened.
fn open_settings_window() -> (window::Id, Task<Message>) {
    let (window_id, task) = window::open(window::Settings {
        size: Size::new(860.0, 610.0),
        resizable: false,
        decorations: false,
        transparent: false,
        visible: true,
        position: window::Position::Centered,
        ..Default::default()
    });
    (window_id, task.map(Message::WindowOpened))
}

/// Helper to open a simple info window (centered, non-resizable).
/// Returns the window ID and task mapped to Message::WindowOpened.
fn open_info_window(size: Size) -> (window::Id, Task<Message>) {
    let (window_id, task) = window::open(window::Settings {
        size,
        resizable: false,
        decorations: false,
        transparent: false,
        visible: true,
        position: window::Position::Centered,
        ..Default::default()
    });
    (window_id, task.map(Message::WindowOpened))
}

/// Helper to close a window if the window_id is Some.
fn close_window_if_some(window_id: Option<window::Id>) -> Task<Message> {
    window_id.map_or_else(Task::none, window::close)
}

/// Open settings window if not already open, setting error message and modal state.
/// Returns the task if window was opened, otherwise Task::none().
fn open_settings_if_needed(app: &mut App, error_msg: String) -> Task<Message> {
    let task = if app.settings_window_id.is_none() {
        let (window_id, task) = open_settings_window();
        app.settings_window_id = Some(window_id);
        app.show_settings_modal = true;
        task
    } else {
        Task::none()
    };

    app.error_message = Some(error_msg);
    task
}

/// Process text: send to cleanup API if enabled, otherwise return task to initialize TTS directly.
/// Sets loading state before returning.
fn process_text_for_tts(
    app: &mut App,
    text: String,
    context: &'static str,
) -> Task<Message> {
    if app.text_cleanup_enabled {
        set_loading_state(app, "Processing content...");
        info!(context, "Natural Reading enabled, sending to service");
        Task::perform(
            async move { system::cleanup_text(&text).await },
            Message::TextCleanupResponse,
        )
    } else {
        set_loading_state(app, "Synthesizing voice...");
        info!(context, "Initializing TTS directly");
        initialize_tts_async(app.selected_backend, text, context, app.selected_polly_voice.clone())
    }
}

/// Initialize TTS provider and start speaking with the given text asynchronously.
/// Returns a Task that will complete when synthesis is done.
/// This prevents blocking the UI thread during TTS synthesis.
fn initialize_tts_async(
    backend: TTSBackend,
    text: String,
    context: &'static str,
    polly_voice_id: Option<String>,
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
        TTSBackend::AwsPolly => {
            // Use provided voice ID or fall back to config/default
            let voice_id = polly_voice_id.or_else(|| config::load_selected_polly_voice());
            PollyTTSProvider::new(voice_id).map(|p| Box::new(p) as Box<dyn TTSProvider>)
        }
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
                info!(text = %text, "Synthesizing text");
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
            let Some(ref mut provider) = app.provider else {
                warn!("PlayPause received with no active provider");
                return Task::none();
            };
            
            match app.playback_state {
                PlaybackState::Playing => {
                    if let Err(e) = provider.pause() {
                        error!(error = %e, "Failed to pause playback");
                    } else {
                        app.playback_state = PlaybackState::Paused;
                        info!("Playback paused");
                    }
                }
                PlaybackState::Paused => {
                    if let Err(e) = provider.resume() {
                        error!(error = %e, "Failed to resume playback");
                    } else {
                        app.playback_state = PlaybackState::Playing;
                        info!("Playback resumed");
                    }
                }
                PlaybackState::Stopped => {}
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
            clear_loading_state(app);
            info!("Playback stopped, closing main window");
            window::latest().and_then(window::close)
        }
        Message::Tick => {
            // Handle loading animation (for TTS or voice downloads)
            if app.is_loading || app.downloading_voice.is_some() {
                app.loading_animation_time += 0.15; // Increment animation time (faster animation)
                if app.loading_animation_time > std::f32::consts::PI * 2.0 {
                    app.loading_animation_time -= std::f32::consts::PI * 2.0;
                }
                
                // Generate animated bar values using sine waves (only for TTS loading, not voice downloads)
                if app.is_loading {
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
                }
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
            if app.settings_window_id.is_some() {
                debug!("Settings window already open, ignoring request");
                return Task::none();
            }
            
            debug!("Settings clicked");
            let (window_id, task) = open_settings_window();
            debug!(?window_id, "Opening settings window");
            app.settings_window_id = Some(window_id);
            app.show_settings_modal = true;
            task
        }
        Message::CloseSettings => {
            app.show_settings_modal = false;
            close_window_if_some(app.settings_window_id.take())
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
                        // Fetch AWS voices if not already loaded
                        if app.polly_voices.is_none() {
                            return Task::perform(
                                async {
                                    crate::voices::aws::fetch_polly_voices().await
                                },
                                Message::PollyVoicesLoaded,
                            );
                        }
                    }
                    Err(e) => {
                        app.error_message = Some(e);
                        warn!("AWS credentials not found when selecting AWS Polly");
                        // Clear voices if credentials are not available
                        app.polly_voices = None;
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
        Message::TextCleanupToggled(enabled) => {
            info!(?enabled, "Natural Reading toggled");
            app.text_cleanup_enabled = enabled;
            // Persist the setting
            config::save_text_cleanup_enabled(enabled);
            Task::none()
        }
        Message::WindowOpened(id) => {
            info!(?id, "Window opened event received");
            if app.main_window_id.is_none() {
                app.main_window_id = Some(id);
                info!("Main window ID set - UI is now visible");
                
                // If we already have pending text (from async fetch), initialize TTS now
                if let Some(text) = app.pending_text.take() {
                    return process_text_for_tts(app, text, "WindowOpened");
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
            if app.voice_selection_window_id == Some(id) {
                app.voice_selection_window_id = None;
            }
            if app.polly_info_window_id == Some(id) {
                app.polly_info_window_id = None;
            }
            if app.screenshot_window_id == Some(id) {
                app.screenshot_window_id = None;
            }
            if app.ocr_info_window_id == Some(id) {
                app.ocr_info_window_id = None;
            }
            if app.text_cleanup_info_window_id == Some(id) {
                app.text_cleanup_info_window_id = None;
            }
            if app.extracted_text_dialog_window_id == Some(id) {
                app.extracted_text_dialog_window_id = None;
                app.extracted_text = None;
                app.extracted_text_editor = None;
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
            if let Some(ref t) = text {
                info!(bytes = t.len(), preview = %t.chars().take(50).collect::<String>(), "Text selected");
            } else {
                info!("No text selected - app will wait for text or close");
            }
            
            // Initialize TTS if window is already open, otherwise store for later
            if let Some(window_id) = app.main_window_id {
                if let Some(text) = text {
                    return process_text_for_tts(app, text, "SelectedTextFetched");
                }
                warn!("No text selected - closing window");
                return window::close(window_id);
            }
            
            // Window not ready yet, store text for WindowOpened handler
            app.pending_text = text;
            trace!("Window not ready yet, text stored for later initialization");
            Task::none()
        }
        Message::TextCleanupResponse(result) => {
            match result {
                Ok(cleaned_text) => {
                    info!(bytes = cleaned_text.len(), "Natural Reading successful, initializing TTS");
                    // Update status to show we're now synthesizing
                    app.status_text = Some("Synthesizing voice...".to_string());
                    return initialize_tts_async(app.selected_backend, cleaned_text, "TextCleanupResponse", app.selected_polly_voice.clone());
                }
                Err(e) => {
                    error!(error = %e, "Natural Reading service failed");
                    clear_loading_state(app);
                    return open_settings_if_needed(app, e);
                }
            }
        }
        Message::TTSInitialized(result) => {
            // Clear loading state regardless of result
            clear_loading_state(app);
            
            match result {
                Ok(()) => {
                    // Retrieve provider from static storage
                    let Ok(mut guard) = PENDING_PROVIDER.lock() else {
                        error!("Failed to lock PENDING_PROVIDER mutex");
                        app.error_message = Some("Internal error: mutex lock failed".to_string());
                        return Task::none();
                    };
                    
                    let Some(send_provider) = guard.take() else {
                        error!("TTS initialization succeeded but no provider found in storage");
                        app.error_message = Some("Internal error: provider not found".to_string());
                        return Task::none();
                    };
                    
                    app.provider = Some(send_provider.0);
                    app.playback_state = PlaybackState::Playing;
                    app.error_message = None;
                    info!("TTS provider initialized and playback started");
                }
                Err(e) => {
                    error!(error = %e, "TTS initialization failed");
                    return open_settings_if_needed(app, e);
                }
            }
            Task::none()
        }
        Message::StartDrag => {
            if let Some(id) = app.main_window_id {
                window::drag(id)
            } else {
                Task::none()
            }
        }
        Message::VoicesJsonLoaded(result) => {
            match result {
                Ok(voices) => {
                    info!(count = voices.len(), "Voices.json loaded successfully");
                    app.voices = Some(voices);
                }
                Err(e) => {
                    error!(error = %e, "Failed to load voices.json");
                    // Show error to user in settings window if it's open
                    if app.settings_window_id.is_some() {
                        app.error_message = Some(format!("Failed to load voices: {}. Check your internet connection.", e));
                    }
                }
            }
            Task::none()
        }
        Message::PollyVoicesLoaded(result) => {
            match result {
                Ok(voices) => {
                    info!(count = voices.len(), "AWS Polly voices loaded successfully");
                    app.polly_voices = Some(voices);
                }
                Err(e) => {
                    debug!(error = %e, "Failed to load AWS Polly voices (credentials may not be configured)");
                    // Don't show error for missing credentials - this is expected if user hasn't configured AWS
                    app.polly_voices = None;
                }
            }
            Task::none()
        }
        Message::OpenVoiceSelection(lang_code) => {
            if app.voice_selection_window_id.is_some() {
                debug!("Voice selection window already open, ignoring request");
                return Task::none();
            }
            
            debug!(language = %lang_code, "Opening voice selection window");
            app.selected_language = Some(lang_code);
            
            let (window_id, task) = window::open(window::Settings {
                size: Size::new(400.0, 500.0), // 33% narrower: 600 * 0.67 ≈ 400
                resizable: false,
                decorations: false,
                transparent: false,
                visible: true,
                position: window::Position::Centered,
                ..Default::default()
            });
            app.voice_selection_window_id = Some(window_id);
            task.map(Message::WindowOpened)
        }
        Message::CloseVoiceSelection => {
            close_window_if_some(app.voice_selection_window_id.take())
        }
        Message::OpenPollyInfo => {
            if app.polly_info_window_id.is_some() {
                debug!("Polly info window already open, ignoring request");
                return Task::none();
            }
            
            debug!("Opening AWS Polly pricing info window");
            let (window_id, task) = open_info_window(Size::new(500.0, 400.0));
            app.polly_info_window_id = Some(window_id);
            task
        }
        Message::ClosePollyInfo => {
            close_window_if_some(app.polly_info_window_id.take())
        }
        Message::OpenPollyPricingUrl => {
            let url = "https://aws.amazon.com/polly/pricing/";
            open_url(url);
            info!("Opening AWS Polly pricing URL in browser");
            Task::none()
        }
        Message::OCRBackendSelected(backend) => {
            // Prevent selection of BetterOCR since it's not available yet
            if backend == OCRBackend::BetterOCR {
                debug!("Better OCR is not available yet, ignoring selection");
                return Task::none();
            }
            info!(?backend, "OCR backend selected");
            app.selected_ocr_backend = backend;
            config::save_ocr_backend(backend);
            Task::none()
        }
        Message::OpenOCRInfo => {
            if app.ocr_info_window_id.is_some() {
                debug!("OCR info window already open, ignoring request");
                return Task::none();
            }
            
            debug!("Opening Better OCR info window");
            let (window_id, task) = open_info_window(Size::new(500.0, 300.0));
            app.ocr_info_window_id = Some(window_id);
            task
        }
        Message::CloseOCRInfo => {
            close_window_if_some(app.ocr_info_window_id.take())
        }
        Message::OpenTextCleanupInfo => {
            if app.text_cleanup_info_window_id.is_some() {
                debug!("Natural Reading info window already open, ignoring request");
                return Task::none();
            }
            
            debug!("Opening Natural Reading info window");
            let (window_id, task) = open_info_window(Size::new(500.0, 300.0));
            app.text_cleanup_info_window_id = Some(window_id);
            task
        }
        Message::CloseTextCleanupInfo => {
            close_window_if_some(app.text_cleanup_info_window_id.take())
        }
        Message::VoiceSelected(voice_key) => {
            info!(voice = %voice_key, "Voice selected");
            match app.selected_backend {
                TTSBackend::Piper => {
                    app.selected_voice = Some(voice_key.clone());
                    config::save_selected_voice(voice_key);
                }
                TTSBackend::AwsPolly => {
                    app.selected_polly_voice = Some(voice_key.clone());
                    config::save_selected_polly_voice(voice_key);
                }
            }
            close_window_if_some(app.voice_selection_window_id.take())
        }
        Message::VoiceDownloadRequested(voice_key) => {
            info!(voice = %voice_key, "Voice download requested");
            
            let voice_info = app.voices.as_ref()
                .and_then(|voices| voices.get(&voice_key).cloned());
            
            if let Some(voice_info) = voice_info {
                // Set downloading state
                app.downloading_voice = Some(voice_key.clone());
                set_loading_state(app, &format!("Downloading voice: {}...", voice_info.name));
                
                // Start async download
                Task::perform(
                    async move {
                        use crate::voices::download;
                        download::download_voice(&voice_key, &voice_info)
                            .await
                            .map(|_| voice_key)
                    },
                    Message::VoiceDownloaded,
                )
            } else {
                error!(voice = %voice_key, "Voice not found in voices.json");
                app.error_message = Some(format!("Voice {} not found", voice_key));
                Task::none()
            }
        }
        Message::VoiceDownloaded(result) => {
            clear_loading_state(app);
            app.downloading_voice = None;
            match result {
                Ok(voice_key) => {
                    info!(voice = %voice_key, "Voice downloaded successfully");
                    app.status_text = Some("Voice downloaded successfully".to_string());
                    // Auto-select the downloaded voice
                    app.selected_voice = Some(voice_key.clone());
                    config::save_selected_voice(voice_key);
                }
                Err(e) => {
                    error!(error = %e, "Voice download failed");
                    app.error_message = Some(format!("Download failed: {}", e));
                }
            }
            Task::none()
        }
        Message::ScreenshotRequested => {
            info!("Screenshot button clicked, starting region selection");
            // Spawn async task to capture screenshot region
            Task::perform(
                async {
                    debug!("Starting async screenshot capture task");
                    // Use spawn_blocking for the blocking shell command
                    let result = tokio::task::spawn_blocking(|| {
                        debug!("Executing capture_region in blocking thread");
                        crate::system::capture_region()
                    })
                    .await;
                    debug!("Screenshot capture task completed");
                    result.unwrap_or_else(|e| {
                        tracing::warn!(error = %e, "Failed to join blocking task for screenshot capture");
                        Err(format!("Task join error: {}", e))
                    })
                },
                Message::ScreenshotCaptured,
            )
        }
        Message::ScreenshotCaptured(result) => {
            match result {
                Ok(file_path) => {
                    info!(path = %file_path, "Screenshot captured successfully");
                    app.screenshot_path = Some(file_path.clone());
                    app.status_text = Some("Extracting text from image...".to_string());
                    
                    // Automatically extract text from the screenshot
                    let file_path_clone = file_path.clone();
                    Task::perform(
                        async move {
                            debug!("Starting async text extraction from screenshot");
                            // Use spawn_blocking for the blocking shell command
                            let result = tokio::task::spawn_blocking(move || {
                                debug!("Executing extract_text_from_image in blocking thread");
                                crate::system::extract_text_from_image(&file_path_clone)
                            })
                            .await;
                            debug!("Text extraction task completed");
                            result.unwrap_or_else(|e| {
                                tracing::warn!(error = %e, "Failed to join blocking task for text extraction");
                                Err(format!("Task join error: {}", e))
                            })
                        },
                        Message::ScreenshotTextExtracted,
                    )
                }
                Err(e) => {
                    // Don't show error for user cancellation
                    if e.contains("cancelled") {
                        debug!("User cancelled screenshot selection");
                    } else {
                        error!(error = %e, "Screenshot capture failed");
                        app.error_message = Some(format!("Screenshot failed: {}", e));
                    }
                    Task::none()
                }
            }
        }
        Message::ScreenshotTextExtracted(result) => {
            match result {
                Ok(extracted_text) => {
                    info!(bytes = extracted_text.len(), "Text extracted from screenshot successfully");
                    info!(
                        text = %extracted_text,
                        "Extracted text from screenshot"
                    );
                    app.status_text = Some("Text extracted from image".to_string());
                    
                    // Store extracted text and initialize editor content
                    app.extracted_text = Some(extracted_text.clone());
                    app.extracted_text_editor = Some(iced::widget::text_editor::Content::with_text(&extracted_text));
                    
                    // Open the extracted text dialog window
                    if app.extracted_text_dialog_window_id.is_none() {
                        let (window_id, task) = window::open(window::Settings {
                            size: Size::new(600.0, 400.0),
                            resizable: true,
                            decorations: true,
                            transparent: false,
                            visible: true,
                            position: window::Position::Centered,
                            ..Default::default()
                        });
                        app.extracted_text_dialog_window_id = Some(window_id);
                        return task.map(Message::WindowOpened);
                    }
                }
                Err(e) => {
                    warn!(error = %e, "Failed to extract text from screenshot");
                    // Don't show error if no text was found (image might not contain text)
                    if e.contains("No text found") {
                        app.status_text = Some("No text found in image".to_string());
                    } else {
                        app.error_message = Some(format!("Text extraction failed: {}", e));
                        app.status_text = Some("Text extraction failed".to_string());
                    }
                }
            }
            Task::none()
        }
        Message::OpenScreenshotViewer => {
            if app.screenshot_window_id.is_some() {
                debug!("Screenshot window already open, ignoring request");
                return Task::none();
            }
            
            if app.screenshot_path.is_none() {
                debug!("No screenshot available to display");
                return Task::none();
            }
            
            debug!("Opening screenshot viewer window");
            let (window_id, task) = window::open(window::Settings {
                size: Size::new(800.0, 600.0),
                resizable: true,
                decorations: true,
                transparent: false,
                visible: true,
                position: window::Position::Centered,
                ..Default::default()
            });
            app.screenshot_window_id = Some(window_id);
            task.map(Message::WindowOpened)
        }
        Message::CloseScreenshotViewer => {
            close_window_if_some(app.screenshot_window_id.take())
        }
        Message::OpenExtractedTextDialog => {
            if app.extracted_text_dialog_window_id.is_some() {
                debug!("Extracted text dialog already open, ignoring request");
                return Task::none();
            }
            
            if app.extracted_text.is_none() {
                debug!("No extracted text available to display");
                return Task::none();
            }
            
            debug!("Opening extracted text dialog window");
            let (window_id, task) = window::open(window::Settings {
                size: Size::new(600.0, 400.0),
                resizable: true,
                decorations: true,
                transparent: false,
                visible: true,
                position: window::Position::Centered,
                ..Default::default()
            });
            app.extracted_text_dialog_window_id = Some(window_id);
            task.map(Message::WindowOpened)
        }
        Message::CloseExtractedTextDialog => {
            app.extracted_text = None;
            app.extracted_text_editor = None;
            close_window_if_some(app.extracted_text_dialog_window_id.take())
        }
        Message::CopyExtractedTextToClipboard => {
            let text_to_copy = app.extracted_text_editor.as_ref()
                .map(|e| e.text())
                .or_else(|| app.extracted_text.clone());
            
            let Some(text_to_copy) = text_to_copy else {
                warn!("No extracted text available to copy");
                return Task::none();
            };
            
            match system::copy_to_clipboard(&text_to_copy) {
                Ok(()) => {
                    info!(bytes = text_to_copy.len(), "Text copied to clipboard successfully");
                    app.status_text = Some("Text copied to clipboard".to_string());
                }
                Err(e) => {
                    error!(error = %e, "Failed to copy text to clipboard");
                    app.error_message = Some(format!("Failed to copy to clipboard: {}", e));
                }
            }
            Task::none()
        }
        Message::ExtractedTextEditorAction(action) => {
            // Apply the action to the editor content
            if let Some(ref mut editor_content) = app.extracted_text_editor {
                editor_content.perform(action);
                // Update the extracted_text string for consistency
                app.extracted_text = Some(editor_content.text());
            }
            Task::none()
        }
        Message::ReadExtractedText => {
            let text_to_read = app.extracted_text_editor.as_ref()
                .map(|e| e.text())
                .or_else(|| app.extracted_text.clone());
            
            let Some(text_to_read) = text_to_read else {
                warn!("No extracted text available to read");
                return Task::none();
            };
            
            if text_to_read.trim().is_empty() {
                warn!("Extracted text is empty, cannot read");
                return Task::none();
            }
            
            info!(bytes = text_to_read.len(), "Sending extracted text to TTS");
            // Don't close the dialog - keep it open so user can edit and read again if needed
            // Process text for TTS (this will handle Natural Reading if enabled)
            process_text_for_tts(app, text_to_read, "ReadExtractedText")
        }
    }
}

