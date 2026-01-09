//! Iced application adapter (thin UI layer)

use iced::time::{self, Duration};
use iced::{Element, Point, Size, Subscription, Task};
use iced::window;
use tracing::{debug, info};

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
        position: window::Position::SpecificWith(|window_size, monitor_size| {
            // Position at bottom-left corner with small margin
            let margin = 20.0;
            Point::new(
                margin,
                monitor_size.height - window_size.height - margin,
            )
        }),
        ..Default::default()
    });
    let open_task = open_task.map(Message::WindowOpened);
    
    // Fetch selected text asynchronously after UI appears (non-blocking)
    // This runs in a background task so it doesn't delay the UI
    let fetch_text_task = Task::perform(
        async {
            debug!("Starting async text fetch task");
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
    
    // Fetch voices.json asynchronously on startup
    let fetch_voices_task = Task::perform(
        async {
            debug!("Fetching voices.json from Hugging Face");
            crate::voices::fetch_voices_json().await
        },
        Message::VoicesJsonLoaded,
    );
    
    (app, Task::batch([open_task, fetch_text_task, fetch_voices_task]))
}

pub fn title(app: &App, window: window::Id) -> String {
    // Set different titles for different windows
    if app.settings_window_id == Some(window) {
        String::from("Settings")
    } else if app.voice_selection_window_id == Some(window) {
        String::from("Select Voice")
    } else {
        String::from("Insight Reader")
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
    
    // Show voice selection window if this is the voice selection window
    if app.voice_selection_window_id == Some(window) {
        return view::voice_selection_window_view(app);
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
    // Poll when playing, paused, loading, or downloading a voice
    let tick = match (app.playback_state, app.is_loading, app.downloading_voice.is_some()) {
        (PlaybackState::Stopped, false, false) => Subscription::none(),
        _ => time::every(Duration::from_millis(75)).map(|_| Message::Tick),
    };
    
    Subscription::batch(vec![window_opened, window_closed, tick])
}
