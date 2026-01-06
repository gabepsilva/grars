//! UI rendering logic

use iced::widget::{button, column, container, progress_bar, row, svg, text, Space};
use iced::theme::Button;
use iced::{Alignment, Color, Element, Length};

use crate::model::{App, Message, PlaybackState};
use crate::styles::{CircleButtonStyle, WaveBarStyle, WindowStyle};

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
            .center_x()
            .center_y(),
    )
    .width(Length::Fixed(36.0))
    .height(Length::Fixed(36.0))
    .style(Button::Custom(Box::new(CircleButtonStyle)))
    .on_press(msg)
    .into()
}

/// Helper to create an SVG icon element.
fn icon(path: &str, size: f32) -> svg::Svg {
    svg(svg::Handle::from_path(path))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
}

pub fn view(app: &App) -> Element<'_, Message> {
    // Waveform visualization using frequency bands
    let waveform: Element<Message> = row((0..NUM_BARS)
        .map(|i| {
            let amplitude = app.frequency_bands.get(i).copied().unwrap_or(0.0);
            let height = bar_height(amplitude);
            container(Space::new(Length::Fixed(3.0), Length::Fixed(height)))
                .style(iced::theme::Container::Custom(Box::new(WaveBarStyle)))
                .into()
        })
        .collect::<Vec<Element<Message>>>())
    .spacing(4)
    .align_items(Alignment::Center)
    .into();

    // Play/pause icon path based on state
    let play_pause_icon = if app.playback_state == PlaybackState::Playing {
        "assets/icons/pause.svg"
    } else {
        "assets/icons/play.svg"
    };

    let controls = row![
        circle_button(
            text("-5s").size(12).style(Color::WHITE),
            Message::SkipBackward
        ),
        circle_button(
            text("+5s").size(12).style(Color::WHITE),
            Message::SkipForward
        ),
        circle_button(icon(play_pause_icon, 16.0), Message::PlayPause),
        circle_button(icon("assets/icons/stop.svg", 16.0), Message::Stop),
    ]
    .spacing(6)
    .align_items(Alignment::Center);

    let main_bar = row![
        icon("assets/icons/volume.svg", 32.0),
        Space::with_width(Length::Fixed(16.0)),
        waveform,
        Space::with_width(Length::Fixed(16.0)),
        controls,
    ]
    .padding([8, 0, 4, 0])
    .align_items(Alignment::Center);

    let progress = container(
        progress_bar(0.0..=1.0, app.progress).height(Length::Fixed(2.0)),
    )
    .padding([8, 0, 8, 0]);

    // Centered content (main controls + progress bar)
    let centered_content = container(
        column![
            container(main_bar).padding([0, 10, 0, 10]),
            container(progress).padding([0, 10, 0, 10]),
        ]
        .width(Length::Shrink)
        .align_items(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x()
    .center_y();

    // Settings icon (independent, top-right) - 32px total width with padding
    let settings_icon = container(icon("assets/icons/settings.svg", 16.0)).padding(8);

    // Layout: [spacer | centered_content | gear]
    // Left spacer balances gear width so content is truly centered
    container(
        row![
            Space::with_width(Length::Fixed(32.0)), // Balance for gear
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
    .style(iced::theme::Container::Custom(Box::new(WindowStyle)))
    .into()
}
