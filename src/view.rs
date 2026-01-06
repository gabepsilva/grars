//! UI rendering logic

use iced::widget::{button, column, container, progress_bar, row, svg, text, Space};
use iced::{Alignment, Color, Element, Length};

use crate::model::{App, Message, PlaybackState};
use crate::styles::{
    circle_button_style, modal_content_style, transparent_button_style, wave_bar_style,
    window_style,
};

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
    let close_button = button(
        container(white_text("✕", 20))
            .width(Length::Fixed(32.0))
            .height(Length::Fixed(32.0))
            .center_x(Length::Fixed(32.0))
            .center_y(Length::Fixed(32.0)),
    )
    .style(circle_button_style)
    .on_press(Message::CloseSettings);

    container(
        column![
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
///
/// Layout structure (window is 380×70):
/// ┌──────────────────────────────────────────────────────┐
/// │  [vol] ||||||||  [-5s] [+5s] [▶] [■]          [⚙]   │
/// │  ════════════════════════════════════════════════    │
/// └──────────────────────────────────────────────────────┘
pub fn main_view(app: &App) -> Element<'_, Message> {
    // 1. Waveform: 10 vertical bars
    let waveform: Element<Message> = row((0..NUM_BARS)
        .map(|i| {
            let amplitude = app.frequency_bands.get(i).copied().unwrap_or(0.0);
            let height = bar_height(amplitude);
            container(
                Space::new()
                    .width(Length::Fixed(3.0))
                    .height(Length::Fixed(height)),
            )
            .style(wave_bar_style)
            .into()
        })
        .collect::<Vec<Element<Message>>>())
    .spacing(4)
    .align_y(Alignment::Center)
    .into();

    // 2. Play/pause icon
    let play_pause_icon = if app.playback_state == PlaybackState::Playing {
        "assets/icons/pause.svg"
    } else {
        "assets/icons/play.svg"
    };

    // 3. Control buttons row
    let controls = row![
        circle_button(white_text("-5s", 12), Message::SkipBackward),
        circle_button(white_text("+5s", 12), Message::SkipForward),
        circle_button(icon(play_pause_icon, 16.0), Message::PlayPause),
        circle_button(icon("assets/icons/stop.svg", 16.0), Message::Stop),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    // 4. Base content row (without gear): [volume] [waveform] [controls]
    let content_row = row![
        icon("assets/icons/volume.svg", 28.0),
        Space::new().width(Length::Fixed(12.0)),
        waveform,
        Space::new().width(Length::Fixed(12.0)),
        controls,
    ]
    .align_y(Alignment::Center)
    .padding([8.0, 16.0]);

    // 5. Progress bar directly under the content row (not under gear)
    let progress = container(progress_bar(0.0..=1.0, app.progress))
        .width(Length::Fixed(313.0))
        .height(Length::Fixed(1.0))
        .padding([0.0, 19.0]);

    let content_column = column![
        content_row,
        Space::new().height(Length::Fixed(3.0)), // small gap
        progress,
    ]
    .width(Length::Shrink);

    // 6. Settings gear (transparent button) on the right
    let settings_btn = button(icon("assets/icons/settings.svg", 18.0))
        .style(transparent_button_style)
        .padding([0.0, 0.0])
        .on_press(Message::Settings);

    // 7. Final row: [content_column | spacer | gear], centered with padding
    let content = row![
        content_column,
        Space::new().width(Length::Fill),
        settings_btn,
    ]
    .align_y(Alignment::Center)
    .padding([4.0, 10.0]); // [top/bottom, left/right]

    // 8. Outer container with window styling
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(window_style)
        .into()
}
