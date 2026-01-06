//! Entry point and window configuration

mod app;
mod model;
mod providers;
mod styles;
mod system;
mod update;
mod view;

use iced::{application, Size};

fn main() -> iced::Result {
    // Read selected text at startup
    let selected_text = crate::system::get_selected_text();

    if let Some(ref text) = selected_text {
        eprintln!("Text Selected: {} bytes", text.len());
    } else {
        eprintln!("No text selected");
    }

    // Store selected text for later initialization after window appears
    crate::app::set_initial_text(selected_text);
    
    // Start the application immediately - window will appear right away
    application(
        crate::app::new,
        crate::app::update,
        crate::app::view
    )
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
