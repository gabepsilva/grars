/// Entry point and window configuration

mod app;
mod model;
mod providers;
mod styles;
mod system;
mod update;
mod view;

use iced::{Application, Settings, Size};

use crate::model::App;
use crate::providers::{PiperTTSProvider, TTSProvider};

fn main() -> iced::Result {
    // Read selected text at startup
    let selected_text = crate::system::get_selected_text();

    if let Some(ref text) = selected_text {
        eprintln!("Text Selected: {} bytes", text.len());
    } else {
        eprintln!("No text selected");
    }

    // Initialize TTS provider and speak selected text
    if let Some(text) = selected_text {
        match PiperTTSProvider::new() {
            Ok(mut provider) => {
                eprintln!("Piper TTS initialized, speaking...");
                if let Err(e) = provider.speak(&text) {
                    eprintln!("TTS error: {e}");
                }
                // Keep provider alive by leaking it (temporary for testing)
                // The audio will play in the background
                Box::leak(Box::new(provider));
            }
            Err(e) => {
                eprintln!("Failed to initialize Piper TTS: {e}");
            }
        }
    }

    App::run(Settings {
        window: iced::window::Settings {
            size: Size::new(380.0, 70.0),
            resizable: false,
            decorations: false,
            transparent: true,
            ..Default::default()
        },
        ..Default::default()
    })
}
