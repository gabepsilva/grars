//! Iced application adapter (thin UI layer)

use iced::time::{self, Duration};
use iced::{Element, Size, Subscription, Task};
use iced::window;
use tracing::info;

use crate::model::{App, Message, PlaybackState};
use crate::update;
use crate::view;

pub fn new() -> (App, Task<Message>) {
    // Create app immediately without waiting for anything
    let app = App::new(None);
    
    info!("App created, opening UI immediately");
    
    // Open the main window (daemon doesn't open one by default)
    // This happens synchronously but is very fast - just window creation
    let (_main_window_id, open_task) = window::open(window::Settings {
        size: Size::new(360.0, 70.0),
        resizable: false,
        decorations: false,
        transparent: true,
        visible: true,
        ..Default::default()
    });
    let open_task = open_task.map(Message::WindowOpened);
    
    // Fetch selected text asynchronously after UI appears (non-blocking)
    // This runs in a background task so it doesn't delay the UI
    let fetch_text_task = Task::perform(
        async {
            use tracing::debug;
            debug!("Starting async text fetch task");
            // Small delay to ensure window is fully visible before fetching
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            debug!("Delay complete, fetching selected text");
            // Use spawn_blocking for the blocking shell command
            let result = tokio::task::spawn_blocking(|| {
                debug!("Executing get_selected_text in blocking thread");
                crate::system::get_selected_text()
            })
            .await;
            debug!("Text fetch task completed");
            result.unwrap_or_else(|e| {
                tracing::warn!(error = %e, "Failed to join blocking task for text fetch");
                None
            })
        },
        Message::SelectedTextFetched,
    );
    
    (app, Task::batch([open_task, fetch_text_task]))
}

pub fn title(app: &App, window: window::Id) -> String {
    // Set different titles for different windows
    if app.settings_window_id == Some(window) {
        String::from("Settings")
    } else {
        String::from("Speaking...")
    }
}

pub fn update(app: &mut App, message: Message) -> Task<Message> {
    update::update(app, message)
}

pub fn view(app: &App, window: window::Id) -> Element<'_, Message> {
    // Show settings window if this is the settings window
    if app.settings_window_id == Some(window) {
        return view::settings_window_view(app);
    }
    
    view::main_view(app)
}

pub fn subscription(app: &App) -> Subscription<Message> {
    // Subscribe to window open/close events
    let window_opened = window::open_events().map(|id| {
        Message::WindowOpened(id)
    });
    
    let window_closed = window::close_events().map(|id| {
        Message::WindowClosed(id)
    });
    
    // Run animation/polling at ~75ms intervals
    // Only poll when playing (not stopped)
    let tick = match app.playback_state {
        PlaybackState::Stopped => Subscription::none(),
        _ => time::every(Duration::from_millis(75)).map(|_| Message::Tick),
    };
    
    Subscription::batch(vec![window_opened, window_closed, tick])
}
