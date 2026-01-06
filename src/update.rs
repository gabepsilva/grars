//! Business logic for state transitions

use iced::window;
use iced::{Size, Task};
use tracing::{debug, error, info, trace, warn};

use crate::config;
use crate::logging;
use crate::model::{App, Message, PlaybackState, TTSBackend};
use crate::providers::{PiperTTSProvider, PollyTTSProvider, TTSProvider};

const SKIP_SECONDS: f32 = 5.0;
const NUM_BANDS: usize = 10;

pub fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::SkipBackward => {
            if let Some(ref mut provider) = app.provider {
                trace!(seconds = SKIP_SECONDS, "Skip backward requested");
                provider.skip_backward(SKIP_SECONDS);
                app.progress = provider.get_progress();
                debug!(progress = app.progress, "Skip backward applied");
            } else {
                warn!("SkipBackward received with no active provider");
            }
            Task::none()
        }
        Message::SkipForward => {
            if let Some(ref mut provider) = app.provider {
                trace!(seconds = SKIP_SECONDS, "Skip forward requested");
                provider.skip_forward(SKIP_SECONDS);
                app.progress = provider.get_progress();
                debug!(progress = app.progress, "Skip forward applied");
            } else {
                warn!("SkipForward received with no active provider");
            }
            Task::none()
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
            info!("Playback stopped, closing main window");
            window::latest().and_then(window::close)
        }
        Message::Tick => {
            if let Some(ref provider) = app.provider {
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
            debug!("Settings clicked");
            let (window_id, task) = window::open(window::Settings {
                size: Size::new(760.0, 280.0),
                resizable: false,
                decorations: true,
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
            debug!(?id, "Window opened");
            if app.main_window_id.is_none() {
                app.main_window_id = Some(id);

                // Initialize TTS provider and start speaking now that window is visible
                if let Some(text) = app.pending_text.take() {
                    info!(
                        backend = ?app.selected_backend,
                        bytes = text.len(),
                        "Initializing TTS provider"
                    );

                    let provider_result: Result<Box<dyn TTSProvider>, _> =
                        match app.selected_backend {
                            TTSBackend::Piper => {
                                PiperTTSProvider::new().map(|p| Box::new(p) as Box<dyn TTSProvider>)
                            }
                            TTSBackend::AwsPolly => {
                                PollyTTSProvider::new().map(|p| Box::new(p) as Box<dyn TTSProvider>)
                            }
                        };

                    match provider_result {
                        Ok(mut provider) => {
                            if let Err(e) = provider.speak(&text) {
                                error!(error = %e, "TTS speak failed");
                            } else {
                                info!("TTS playback started");
                                app.provider = Some(provider);
                                app.playback_state = PlaybackState::Playing;
                            }
                        }
                        Err(e) => {
                            error!(error = %e, "Failed to initialize TTS provider");
                        }
                    }
                }
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
            Task::none()
        }
    }
}
