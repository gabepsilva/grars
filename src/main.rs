//! Entry point and window configuration

mod app;
mod model;
mod providers;
mod styles;
mod system;
mod update;
mod view;

use iced::{application, Size};

use crate::providers::{PiperTTSProvider, TTSProvider};

fn main() -> iced::Result {
    // Read selected text at startup
    let selected_text = crate::system::get_selected_text();

    if let Some(ref text) = selected_text {
        eprintln!("Text Selected: {} bytes", text.len());
    } else {
        eprintln!("No text selected");
    }

    // Initialize TTS provider and start speaking
    let provider = selected_text.and_then(|text| {
        match PiperTTSProvider::new() {
            Ok(mut provider) => {
                if let Err(e) = provider.speak(&text) {
                    eprintln!("TTS error: {e}");
                    return None;
                }
                Some(provider)
            }
            Err(e) => {
                eprintln!("Failed to initialize Piper TTS: {e}");
                None
            }
        }
    });

    // Store provider in a way that can be accessed by the app
    // For now, we'll initialize the app and set provider in the first update
    crate::app::set_initial_provider(provider);
    
    application(
        crate::app::new,
        crate::app::update,
        crate::app::view
    )
        .title(crate::app::title)
        .subscription(crate::app::subscription)
        .window(iced::window::Settings {
            size: Size::new(380.0, 70.0),
            resizable: false,
            decorations: false,
            transparent: true,
            ..Default::default()
        })
        .run()
}
