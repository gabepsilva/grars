//! UI rendering logic

use iced::widget::{button, column, container, progress_bar, row, svg, text, Space};
use iced::{Alignment, Color, Element, Length};

use crate::model::{App, Message, PlaybackState};
use crate::styles::{circle_button_style, modal_content_style, wave_bar_style, window_style};

const MIN_HEIGHT: f32 = 4.0;
const MAX_HEIGHT: f32 = 24.0;
const NUM_BARS: usize = 10;

/// Calculate bar height from frequency band amplitude (0.0-1.0).
fn bar_height(amplitude: f32) -> f32 {
    MIN_HEIGHT + amplitude * (MAX_HEIGHT - MIN_HEIGHT)
}

/// Helper to create a 36x36 circle button with centered content.
fn circle_button<'a>(
    content: impl Into<Element<'a, Message>>,
    msg: Message,
) -> Element<'a, Message> {
    button(
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .width(Length::Fixed(36.0))
    .height(Length::Fixed(36.0))
    .style(circle_button_style)
    .on_press(msg)
    .into()
}

/// Helper to create an SVG icon element.
fn icon(path: &str, size: f32) -> svg::Svg<'_> {
    svg(svg::Handle::from_path(path))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
}

/// Helper to create white text with consistent styling.
fn white_text(content: &str, size: u32) -> text::Text<'_> {
    text(content)
        .size(size)
        .style(|_theme| iced::widget::text::Style {
            color: Some(Color::WHITE),
        })
}

/// Settings window view - floating modal style
pub fn settings_window_view<'a>() -> Element<'a, Message> {
    // Close button in top-right corner
    let close_button = button(
        container(white_text("✕", 20))
            .width(Length::Fixed(32.0))
            .height(Length::Fixed(32.0))
            .center_x(Length::Fixed(32.0))
            .center_y(Length::Fixed(32.0))
    )
    .style(circle_button_style)
    .on_press(Message::CloseSettings);
    
    container(
        column![
            // Header with title and close button
            row![
                white_text("Settings", 24),
                Space::new().width(Length::Fill),
                close_button,
            ]
            .width(Length::Fill)
            .align_y(Alignment::Center),
            Space::new().height(Length::Fixed(20.0)),
            text("Settings content goes here")
                .size(16)
                .style(|_theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.8)),
                }),
            Space::new().height(Length::Fixed(30.0)),
            // Also add a larger close button at the bottom for easier access
            button(white_text("Close Window", 16))
                .style(circle_button_style)
                .padding(16)
                .on_press(Message::CloseSettings),
        ]
        .padding(30)
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(modal_content_style)
    .into()
}

/// Main window view
pub fn main_view(app: &App) -> Element<'_, Message> {
    // Waveform visualization using frequency bands
    let waveform: Element<Message> = row((0..NUM_BARS)
        .map(|i| {
            let amplitude = app.frequency_bands.get(i).copied().unwrap_or(0.0);
            let height = bar_height(amplitude);
            container(Space::new().width(Length::Fixed(3.0)).height(Length::Fixed(height)))
                .style(wave_bar_style)
                .into()
        })
        .collect::<Vec<Element<Message>>>())
    .spacing(4)
    .align_y(Alignment::Center)
    .into();

    // Play/pause icon path based on state
    let play_pause_icon = if app.playback_state == PlaybackState::Playing {
        "assets/icons/pause.svg"
    } else {
        "assets/icons/play.svg"
    };

    let controls = row![
        circle_button(white_text("-5s", 12), Message::SkipBackward),
        circle_button(white_text("+5s", 12), Message::SkipForward),
        circle_button(icon(play_pause_icon, 16.0), Message::PlayPause),
        circle_button(icon("assets/icons/stop.svg", 16.0), Message::Stop),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let main_bar = row![
        icon("assets/icons/volume.svg", 32.0),
        Space::new().width(Length::Fixed(16.0)),
        waveform,
        Space::new().width(Length::Fixed(16.0)),
        controls,
    ]
    .padding(8)
    .align_y(Alignment::Center);

    let progress = container(
        container(progress_bar(0.0..=1.0, app.progress))
            .width(Length::Fill)
            .height(Length::Fixed(2.0)),
    )
    .padding(8);

    // Centered content (main controls + progress bar)
    let centered_content = container(
        column![
            container(main_bar).padding(10),
            container(progress).padding(10),
        ]
        .width(Length::Shrink)
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill);

    // Settings icon (independent, top-right) - 32px total width with padding
    let settings_icon = button(container(icon("assets/icons/settings.svg", 18.0)).padding(4))
        .style(button::text) // Transparent button style
        .on_press(Message::Settings);

    // Layout: [spacer | centered_content | gear]
    // Left spacer balances gear width so content is truly centered
    let main_content = container(
        row![
            Space::new().width(Length::Fixed(32.0)), // Balance for gear
            centered_content,
            container(settings_icon)
                .width(Length::Fixed(32.0))
                .height(Length::Fill)
                .align_y(iced::alignment::Vertical::Top), // Gear at top
        ]
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(window_style);

    main_content.into()
}
