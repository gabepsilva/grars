//! Hotkey configuration UI component

use iced::widget::{button, checkbox, column, container, row, text, Space};
use iced::{Alignment, Color, Element, Length};

use crate::model::Message;
use crate::styles::{circle_button_style, section_style, white_checkbox_style};
use crate::system::{format_hotkey_display};

/// Helper to create white text with consistent styling (matching view.rs pattern).
fn white_text(content: &str, size: u32) -> text::Text<'_> {
    text(content)
        .size(size)
        .style(|_theme| iced::widget::text::Style {
            color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.85)),
        })
}

/// Create the hotkey settings section for the settings window
pub fn hotkey_settings_section<'a>(app: &'a crate::model::App) -> Element<'a, Message> {
    // Format hotkey display string
    let hotkey_display = format_hotkey_display(&app.hotkey_config);
    
    // Check if hotkeys are disabled due to Wayland/Hyprland
    let is_disabled = app.hotkeys_disabled_wayland;
    
    // Hotkey enabled checkbox (disabled if on Wayland/Hyprland)
    let checkbox_label = format!("Enable global hotkey ({})", hotkey_display);
    let mut hotkey_checkbox = checkbox(if is_disabled { false } else { app.hotkey_enabled })
        .label(checkbox_label)
        .style(white_checkbox_style);
    if !is_disabled {
        hotkey_checkbox = hotkey_checkbox.on_toggle(Message::HotkeyToggled);
    }
    
    // Set Hotkey button (disabled if on Wayland/Hyprland)
    let set_button_text = if app.listening_for_hotkey { "Cancel" } else { "Set Hotkey" };
    let mut set_button = button(white_text(set_button_text, 12))
        .style(circle_button_style)
        .padding([6.0, 12.0]);
    if !is_disabled {
        set_button = set_button.on_press(if app.listening_for_hotkey {
            Message::StopListeningForHotkey
        } else {
            Message::StartListeningForHotkey
        });
    }
    
    // Status/info message (listening status or Wayland/Hyprland info)
    let status_message: Option<Element<'a, Message>> = if app.listening_for_hotkey && !is_disabled {
        Some(white_text("Press your key combination...", 11)
            .style(|_theme| iced::widget::text::Style {
                color: Some(Color::from_rgb(0.4, 0.6, 1.0)),
            })
            .into())
    } else if is_disabled {
        Some(
            row![
                white_text("ⓘ", 14)
                    .style(|_theme| iced::widget::text::Style {
                        color: Some(Color::from_rgb(0.3, 0.6, 1.0)),
                    }),
                Space::new().width(Length::Fixed(6.0)),
                white_text("Not supported on Wayland with Hyprland. Please set up key bindings in Hyprland config.", 11)
                    .style(|_theme| iced::widget::text::Style {
                        color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
                    }),
            ]
            .align_y(Alignment::Center)
            .spacing(0)
            .into()
        )
    } else {
        None
    };
    
    let hotkey_control = column![
        row![
            hotkey_checkbox,
            Space::new().width(Length::Fixed(12.0)),
            set_button,
        ]
        .align_y(Alignment::Center)
        .spacing(0),
        if let Some(msg) = status_message {
            column![
                Space::new().height(Length::Fixed(6.0)),
                msg,
            ]
            .spacing(0)
        } else {
            column![].spacing(0)
        },
    ]
    .spacing(0);

    container(
        row![
            container(
                white_text("Global Hotkey", 14)
            )
            .width(Length::Fixed(120.0))
            .align_x(Alignment::Start),
            Space::new().width(Length::Fixed(16.0)),
            container(hotkey_control)
                .width(Length::Fill)
                .align_x(Alignment::Start),
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .padding([12.0, 16.0])
    )
    .style(section_style)
    .into()
}

/// Convert Iced keyboard Key to global_hotkey Code
pub fn iced_key_to_global_hotkey_code(key: &iced::keyboard::Key) -> Option<global_hotkey::hotkey::Code> {
    use global_hotkey::hotkey::Code;
    use iced::keyboard::{key::Named, Key};
    
    match key {
        // Character keys (letters and numbers)
        Key::Character(c) if c.len() == 1 => {
            let ch = c.chars().next()?;
            let ch_upper = ch.to_ascii_uppercase();
            match ch_upper {
                'A' => Some(Code::KeyA),
                'B' => Some(Code::KeyB),
                'C' => Some(Code::KeyC),
                'D' => Some(Code::KeyD),
                'E' => Some(Code::KeyE),
                'F' => Some(Code::KeyF),
                'G' => Some(Code::KeyG),
                'H' => Some(Code::KeyH),
                'I' => Some(Code::KeyI),
                'J' => Some(Code::KeyJ),
                'K' => Some(Code::KeyK),
                'L' => Some(Code::KeyL),
                'M' => Some(Code::KeyM),
                'N' => Some(Code::KeyN),
                'O' => Some(Code::KeyO),
                'P' => Some(Code::KeyP),
                'Q' => Some(Code::KeyQ),
                'R' => Some(Code::KeyR),
                'S' => Some(Code::KeyS),
                'T' => Some(Code::KeyT),
                'U' => Some(Code::KeyU),
                'V' => Some(Code::KeyV),
                'W' => Some(Code::KeyW),
                'X' => Some(Code::KeyX),
                'Y' => Some(Code::KeyY),
                'Z' => Some(Code::KeyZ),
                '0' => Some(Code::Digit0),
                '1' => Some(Code::Digit1),
                '2' => Some(Code::Digit2),
                '3' => Some(Code::Digit3),
                '4' => Some(Code::Digit4),
                '5' => Some(Code::Digit5),
                '6' => Some(Code::Digit6),
                '7' => Some(Code::Digit7),
                '8' => Some(Code::Digit8),
                '9' => Some(Code::Digit9),
                _ => None,
            }
        }
        // Named keys
        Key::Named(named) => match named {
            Named::Space => Some(Code::Space),
            Named::Enter => Some(Code::Enter),
            Named::Tab => Some(Code::Tab),
            Named::Backspace => Some(Code::Backspace),
            Named::Escape => Some(Code::Escape),
            Named::ArrowUp => Some(Code::ArrowUp),
            Named::ArrowDown => Some(Code::ArrowDown),
            Named::ArrowLeft => Some(Code::ArrowLeft),
            Named::ArrowRight => Some(Code::ArrowRight),
            Named::Home => Some(Code::Home),
            Named::End => Some(Code::End),
            Named::PageUp => Some(Code::PageUp),
            Named::PageDown => Some(Code::PageDown),
            Named::Insert => Some(Code::Insert),
            Named::Delete => Some(Code::Delete),
            Named::F1 => Some(Code::F1),
            Named::F2 => Some(Code::F2),
            Named::F3 => Some(Code::F3),
            Named::F4 => Some(Code::F4),
            Named::F5 => Some(Code::F5),
            Named::F6 => Some(Code::F6),
            Named::F7 => Some(Code::F7),
            Named::F8 => Some(Code::F8),
            Named::F9 => Some(Code::F9),
            Named::F10 => Some(Code::F10),
            Named::F11 => Some(Code::F11),
            Named::F12 => Some(Code::F12),
            // Modifier keys - these should not be used as the main key
            Named::Shift | Named::Control | Named::Alt | Named::Super => None,
            _ => None,
        },
        _ => None,
    }
}

/// Convert Iced keyboard Modifiers to global_hotkey Modifiers
pub fn iced_modifiers_to_global_hotkey_modifiers(modifiers: iced::keyboard::Modifiers) -> global_hotkey::hotkey::Modifiers {
    use global_hotkey::hotkey::Modifiers as GHModifiers;
    use iced::keyboard::Modifiers as IcedModifiers;
    
    let mut result = GHModifiers::empty();
    
    if modifiers.contains(IcedModifiers::SHIFT) {
        result |= GHModifiers::SHIFT;
    }
    if modifiers.contains(IcedModifiers::ALT) {
        result |= GHModifiers::ALT;
    }
    if modifiers.contains(IcedModifiers::CTRL) {
        result |= GHModifiers::CONTROL;
    }
    if modifiers.contains(IcedModifiers::LOGO) {
        result |= GHModifiers::META;
    }
    
    result
}
