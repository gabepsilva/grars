/// Custom style functions for UI components (Iced 0.13+ closure-based API)

use iced::widget::{button, container};
use iced::{Color, Theme, Background, Border};

pub fn window_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::BLACK)),
        border: Border {
            color: Color::WHITE,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

pub fn wave_bar_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.6))),
        border: Border {
            radius: 1.5.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn circle_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let base_bg = match status {
        button::Status::Active => Color::from_rgba(1.0, 1.0, 1.0, 0.15),
        button::Status::Hovered => Color::from_rgba(1.0, 1.0, 1.0, 0.25),
        button::Status::Pressed => Color::from_rgba(1.0, 1.0, 1.0, 0.35),
        _ => Color::from_rgba(1.0, 1.0, 1.0, 0.15),
    };
    
    button::Style {
        background: Some(Background::Color(base_bg)),
        text_color: Color::WHITE,
        border: Border {
            radius: 18.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn modal_content_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
        border: Border {
            color: Color::WHITE,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}


