/// Custom style sheets for UI components

use iced::widget::{button, container};
use iced::{Color, Theme};

pub struct WindowStyle;

impl container::StyleSheet for WindowStyle {
    type Style = Theme;
    
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(iced::Background::Color(Color::BLACK)),
            border: iced::Border {
                color: Color::WHITE,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }
}

pub struct WaveBarStyle;

impl container::StyleSheet for WaveBarStyle {
    type Style = Theme;
    
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.6))),
            border: iced::Border {
                radius: 1.5.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

pub struct CircleButtonStyle;

impl button::StyleSheet for CircleButtonStyle {
    type Style = Theme;
    
    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.15))),
            text_color: Color::WHITE,
            border: iced::Border {
                radius: 18.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
    
    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.25))),
            ..self.active(style)
        }
    }
    
    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.35))),
            ..self.active(style)
        }
    }
}


