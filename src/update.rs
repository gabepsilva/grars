//! Business logic for state transitions

use iced::window;
use iced::{Task, Size};

use crate::model::{App, Message, PlaybackState, TTSBackend};
use crate::config;
use crate::providers::{PiperTTSProvider, PollyTTSProvider, TTSProvider};

const SKIP_SECONDS: f32 = 5.0;
const NUM_BANDS: usize = 10;

pub fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::SkipBackward => {
            if let Some(ref mut provider) = app.provider {
                provider.skip_backward(SKIP_SECONDS);
                app.progress = provider.get_progress();
            }
            Task::none()
        }
        Message::SkipForward => {
            if let Some(ref mut provider) = app.provider {
                provider.skip_forward(SKIP_SECONDS);
                app.progress = provider.get_progress();
            }
            Task::none()
        }
        Message::PlayPause => {
            if let Some(ref mut provider) = app.provider {
                match app.playback_state {
                    PlaybackState::Playing => {
                        if provider.pause().is_ok() {
                            app.playback_state = PlaybackState::Paused;
                        }
                    }
                    PlaybackState::Paused => {
                        if provider.resume().is_ok() {
                            app.playback_state = PlaybackState::Playing;
                        }
                    }
                    PlaybackState::Stopped => {}
                }
            }
            Task::none()
        }
        Message::Stop => {
            if let Some(ref mut provider) = app.provider {
                provider.stop().ok();
            }
            app.playback_state = PlaybackState::Stopped;
            app.progress = 0.0;
            app.frequency_bands = vec![0.0; NUM_BANDS];
            window::latest().and_then(window::close)
        }
        Message::Tick => {
            if let Some(ref provider) = app.provider {
                app.progress = provider.get_progress();
                app.frequency_bands = provider.get_frequency_bands(NUM_BANDS);

                if !provider.is_playing() && !provider.is_paused() {
                    app.playback_state = PlaybackState::Stopped;
                    return window::latest().and_then(window::close);
                }
            }
            Task::none()
        }
        Message::Settings => {
            eprintln!("Settings clicked");
            let (window_id, task) = window::open(window::Settings {
                size: Size::new(760.0, 280.0),
                resizable: false,
                decorations: true,
                transparent: false,
                visible: true,
                position: window::Position::Centered,
                ..Default::default()
            });
            eprintln!("Opening settings window with ID: {:?}", window_id);
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
            eprintln!("UI: provider selected = {:?}", backend);
            app.selected_backend = backend;
            // Persist the selected backend so future runs remember the choice.
            config::save_voice_provider(backend);
            Task::none()
        }
        Message::LogLevelSelected(level) => {
            eprintln!("UI: log level selected = {:?}", level);
            app.log_level = level;
            // Persist the selected log level so future runs remember the choice.
            config::save_log_level(level);
            Task::none()
        }
        Message::WindowOpened(id) => {
            eprintln!("Window opened: {:?}", id);
            if app.main_window_id.is_none() {
                app.main_window_id = Some(id);
                
                // Initialize TTS provider and start speaking now that window is visible
                if let Some(text) = app.pending_text.take() {
                    eprintln!(
                        "Initializing {:?} TTS for {} bytes",
                        app.selected_backend,
                        text.len()
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
                                eprintln!("TTS error: {e}");
                            } else {
                                app.provider = Some(provider);
                                app.playback_state = PlaybackState::Playing;
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to initialize TTS: {e}");
                        }
                    }
                }
            }
            app.current_window_id = Some(id);
            Task::none()
        }
        Message::WindowClosed(id) => {
            eprintln!("Window closed: {:?}", id);
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
