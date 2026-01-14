//! UI rendering logic

use iced::widget::{button, checkbox, column, container, mouse_area, progress_bar, radio, row, scrollable, svg, text, text_editor, Space};
use iced::{Alignment, Background, Color, ContentFit, Element, Length};

use crate::flags;
use crate::model::{App, LanguageInfo, LogLevel, Message, OCRBackend, PlaybackState, TTSBackend};
use crate::styles::{
    circle_button_style, close_button_style, error_container_style, header_style,
    modal_content_style, section_style, transparent_button_style, wave_bar_style,
    white_checkbox_style, white_radio_style, window_style,
};

const MIN_HEIGHT: f32 = 4.0;
const MAX_HEIGHT: f32 = 24.0;
const NUM_BARS: usize = 10;

/// Convert AWS Polly engine string to display name
fn engine_display_name(engine: &str) -> &str {
    match engine {
        "Standard" => "Standard",
        "Neural" => "Neural",
        "Generative" => "Generative",
        "LongForm" => "Long-Form",
        _ => engine,
    }
}

// Bundled SVG icons (embedded at compile time)
const SVG_PLAY: &[u8] = include_bytes!("../assets/icons/play.svg");
const SVG_PAUSE: &[u8] = include_bytes!("../assets/icons/pause.svg");
const SVG_STOP: &[u8] = include_bytes!("../assets/icons/stop.svg");
const SVG_VOLUME: &[u8] = include_bytes!("../assets/icons/volume.svg");
const SVG_SETTINGS: &[u8] = include_bytes!("../assets/icons/settings.svg");
const SVG_CAMERA: &[u8] = include_bytes!("../assets/icons/camera.svg");
const SVG_CLIPBOARD: &[u8] = include_bytes!("../assets/icons/clipboard.svg");

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

/// Helper to create an SVG icon element from bundled bytes.
fn icon_from_bytes(bytes: &'static [u8], size: f32) -> svg::Svg<'static> {
    svg(svg::Handle::from_memory(bytes))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
}

/// Icon helper functions for each bundled SVG.
fn play_icon(size: f32) -> svg::Svg<'static> {
    icon_from_bytes(SVG_PLAY, size)
}

fn pause_icon(size: f32) -> svg::Svg<'static> {
    icon_from_bytes(SVG_PAUSE, size)
}

fn stop_icon(size: f32) -> svg::Svg<'static> {
    icon_from_bytes(SVG_STOP, size)
}

fn volume_icon(size: f32) -> svg::Svg<'static> {
    icon_from_bytes(SVG_VOLUME, size)
}

fn settings_icon(size: f32) -> svg::Svg<'static> {
    icon_from_bytes(SVG_SETTINGS, size)
}

fn camera_icon(size: f32) -> svg::Svg<'static> {
    icon_from_bytes(SVG_CAMERA, size)
}

fn clipboard_icon(size: f32) -> svg::Svg<'static> {
    icon_from_bytes(SVG_CLIPBOARD, size)
}

/// Helper to create white text with consistent styling.
fn white_text(content: &str, size: u32) -> text::Text<'_> {
    text(content)
        .size(size)
        .style(|_theme| iced::widget::text::Style {
            color: Some(Color::WHITE),
        })
}

/// Helper to create red error text with consistent styling.
fn error_text(content: &str, size: u32) -> text::Text<'_> {
    text(content)
        .size(size)
        .style(|_theme| iced::widget::text::Style {
            color: Some(Color::from_rgb(1.0, 0.3, 0.3)),
        })
}

/// Helper to create a close button for modal windows.
fn close_button<'a>(msg: Message) -> Element<'a, Message> {
    button(
        container(white_text("✕", 18))
            .width(Length::Fixed(28.0))
            .height(Length::Fixed(28.0))
            .center_x(Length::Fixed(28.0))
            .center_y(Length::Fixed(28.0)),
    )
    .style(close_button_style)
    .on_press(msg)
    .into()
}

/// Helper to create a modal header bar with title and close button.
fn modal_header<'a>(title: &'a str, close_msg: Message) -> Element<'a, Message> {
    container(
        row![
            white_text(title, 20)
                .style(|_theme| iced::widget::text::Style {
                    color: Some(Color::WHITE),
                }),
            Space::new().width(Length::Fill),
            close_button(close_msg),
        ]
        .width(Length::Fill)
        .align_y(Alignment::Center)
    )
    .width(Length::Fill)
    .padding([20.0, 24.0])
    .style(header_style)
    .into()
}

/// Settings window view - floating modal style
pub fn settings_window_view<'a>(app: &'a App) -> Element<'a, Message> {

    // Error message display (if present)
    let error_display: Element<'a, Message> = if let Some(error_msg) = &app.error_message {
        container(
            container(
                error_text(error_msg, 13)
                    .width(Length::Fill)
            )
            .width(Length::Fill)
            .padding(12)
            .style(error_container_style)
        )
        .padding([16, 16]) // Extra top padding to show it's part of the provider section
        .width(Length::Fill)
        .into()
    } else {
        column![].spacing(0).into()
    };

    // TTS Provider section
    let provider_controls = column![
        radio(
            "Piper (offline, CPU)",
            TTSBackend::Piper,
            Some(app.selected_backend),
            Message::ProviderSelected
        )
        .style(white_radio_style),
        Space::new().height(Length::Fixed(6.0)),
        row![
            radio(
                "AWS Polly (Cloud, BYO credentials)",
                TTSBackend::AwsPolly,
                Some(app.selected_backend),
                Message::ProviderSelected
            )
            .style(white_radio_style),
            Space::new().width(Length::Fixed(8.0)),
            // Info icon button (circled i)
            button(
                container(
                    white_text("ⓘ", 16)
                        .style(|_theme| iced::widget::text::Style {
                            color: Some(Color::from_rgb(0.3, 0.6, 1.0)),
                        })
                )
                .width(Length::Fixed(24.0))
                .height(Length::Fixed(24.0))
                .center_x(Length::Fixed(24.0))
                .center_y(Length::Fixed(24.0))
            )
            .style(transparent_button_style)
            .width(Length::Fixed(24.0))
            .height(Length::Fixed(24.0))
            .on_press(Message::OpenPollyInfo),
        ]
        .align_y(Alignment::Center)
        .spacing(0),
    ]
    .spacing(0);

    // AWS Polly error message display (if present and AWS Polly is selected)
    let polly_error_display: Element<'a, Message> = if app.selected_backend == TTSBackend::AwsPolly {
        if let Some(error_msg) = &app.polly_error_message {
            container(
                container(
                    error_text(error_msg, 13)
                        .width(Length::Fill)
                )
                .width(Length::Fill)
                .padding(12)
                .style(error_container_style)
            )
            .padding([12, 16]) // Add padding around the error box
            .width(Length::Fill)
            .into()
        } else {
            column![].spacing(0).into()
        }
    } else {
        column![].spacing(0).into()
    };

    // Piper Voice section (only shown when Piper is selected)
    let piper_voice_section: Element<'a, Message> = if app.selected_backend == TTSBackend::Piper {
        use crate::voices;
        
        // Current voice display
        let current_voice_display = if let Some(ref voice_key) = app.selected_voice {
            text(format!("Piper voice selected: {}", voice_key))
                .size(14)
                .style(|_theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
                })
        } else {
            text("No voice selected")
                .size(14)
                .style(|_theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
                })
        };
        
        // Get available languages from voices - create controls inline to avoid lifetime issues
        let language_controls: Element<'a, Message> = if let Some(ref voices) = app.voices {
            let languages = voices::get_available_languages(voices);
            
            // Create grid layout: 4 columns
            const COLS: usize = 4;
            let mut grid_rows = column![].spacing(6);
            let mut current_row = row![].spacing(8);
            let mut col_count = 0;
            
            // Show all languages in a grid
            for (lang_code, lang_info) in languages.iter() {
                let flag_icon = flags::get_flag_icon(lang_code);
                let label_text = format!("{} ({})", lang_info.name_english, lang_code);
                let lang_code_clone = lang_code.clone();
                let is_selected = app.selected_language.as_deref() == Some(lang_code);
                
                // Create button with SVG flag + text label
                let lang_button = button(
                    container(
                        row![
                            flag_icon,
                            Space::new().width(Length::Fixed(6.0)),
                            text(label_text).size(13)
                                .style(move |_theme| iced::widget::text::Style {
                                    color: Some(if is_selected {
                                        Color::WHITE
                                    } else {
                                        Color::from_rgba(1.0, 1.0, 1.0, 0.7)
                                    }),
                                })
                        ]
                        .align_y(Alignment::Center)
                    )
                    .padding([5.0, 8.0])
                    .width(Length::Fill)
                )
                .style(transparent_button_style)
                .width(Length::Fill)
                .on_press(Message::OpenVoiceSelection(lang_code_clone));
                
                current_row = current_row.push(
                    container(lang_button)
                        .width(Length::Fill)
                );
                col_count += 1;
                
                // Start new row when we reach column limit
                if col_count >= COLS {
                    grid_rows = grid_rows.push(current_row);
                    current_row = row![].spacing(8);
                    col_count = 0;
                }
            }
            
            // Add remaining items in the last row
            if col_count > 0 {
                // Fill remaining columns with empty space
                while col_count < COLS {
                    current_row = current_row.push(
                        container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
                            .width(Length::Fill)
                    );
                    col_count += 1;
                }
                grid_rows = grid_rows.push(current_row);
            }
            
            // Make language grid scrollable (without Select Voice button)
            scrollable(grid_rows)
                .height(Length::Fixed(300.0))
                .into()
        } else {
            // Voices not loaded yet
            column![
                white_text("Loading voices...", 12)
                    .style(|_theme| iced::widget::text::Style {
                        color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.6)),
                    }),
            ]
            .spacing(0)
            .into()
        };
        
        container(
            container(
                column![
                    // Current voice display
                    container(current_voice_display)
                        .width(Length::Fill)
                        .align_x(Alignment::Start)
                        .padding([12.0, 16.0]),
                    // Language grid below
                    container(language_controls)
                        .width(Length::Fill)
                        .padding([0.0, 16.0]),
                ]
                .spacing(0)
            )
            .style(section_style)
        )
        .padding([16, 16]) // Extra top padding to show it's part of the provider section
        .width(Length::Fill)
        .into()
    } else {
        column![].spacing(0).into()
    };

    // AWS Polly Voice section (only shown when AWS Polly is selected and voices are loaded)
    let polly_voice_section: Element<'a, Message> = if app.selected_backend == TTSBackend::AwsPolly {
        use crate::voices::aws;
        
        // Only show if voices are loaded (which means credentials are configured)
        if let Some(ref voices) = app.polly_voices {
            // Current voice display
            let current_voice_display = if let Some(ref voice_key) = app.selected_polly_voice {
                // Parse voice key to show friendly name
                let display_text = if let Some((voice_id, engine_str)) = voice_key.split_once(':') {
                    let engine_display = engine_display_name(engine_str);
                    // Try to get voice name from loaded voices
                    if let Some(voice_info) = voices.get(voice_key) {
                        format!("AWS Polly voice selected: {} ({}, {})", voice_info.name, voice_info.gender, engine_display)
                    } else {
                        format!("AWS Polly voice selected: {} ({})", voice_id, engine_display)
                    }
                } else {
                    // Fallback for old format (just voice ID)
                    format!("AWS Polly voice selected: {}", voice_key)
                };
                text(display_text)
                    .size(14)
                    .style(|_theme| iced::widget::text::Style {
                        color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
                    })
            } else {
                text("No voice selected")
                    .size(14)
                    .style(|_theme| iced::widget::text::Style {
                        color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
                    })
            };
            
            // Get available languages from AWS voices - create controls inline to avoid lifetime issues
            let languages = aws::get_available_languages(voices);
            
            // Create grid layout: 4 columns
            const COLS: usize = 4;
            let mut grid_rows = column![].spacing(6);
            let mut current_row = row![].spacing(8);
            let mut col_count = 0;
            
            // Show all languages in a grid
            for (lang_code, lang_info) in languages.iter() {
                let flag_icon = flags::get_flag_icon(lang_code);
                let label_text = format!("{} ({})", lang_info.name_english, lang_code);
                let lang_code_clone = lang_code.clone();
                let is_selected = app.selected_language.as_deref() == Some(lang_code);
                
                // Create button with SVG flag + text label
                let lang_button = button(
                    container(
                        row![
                            flag_icon,
                            Space::new().width(Length::Fixed(6.0)),
                            text(label_text).size(13)
                                .style(move |_theme| iced::widget::text::Style {
                                    color: Some(if is_selected {
                                        Color::WHITE
                                    } else {
                                        Color::from_rgba(1.0, 1.0, 1.0, 0.7)
                                    }),
                                })
                        ]
                        .align_y(Alignment::Center)
                    )
                    .padding([5.0, 8.0])
                    .width(Length::Fill)
                )
                .style(transparent_button_style)
                .width(Length::Fill)
                .on_press(Message::OpenVoiceSelection(lang_code_clone));
                
                current_row = current_row.push(
                    container(lang_button)
                        .width(Length::Fill)
                );
                col_count += 1;
                
                // Start new row when we reach column limit
                if col_count >= COLS {
                    grid_rows = grid_rows.push(current_row);
                    current_row = row![].spacing(8);
                    col_count = 0;
                }
            }
            
            // Add remaining items in the last row
            if col_count > 0 {
                // Fill remaining columns with empty space
                while col_count < COLS {
                    current_row = current_row.push(
                        container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
                            .width(Length::Fill)
                    );
                    col_count += 1;
                }
                grid_rows = grid_rows.push(current_row);
            }
            
            let language_controls: Element<'a, Message> = scrollable(grid_rows)
                .height(Length::Fixed(300.0))
                .into();
            
            container(
                container(
                    column![
                        // Current voice display
                        container(current_voice_display)
                            .width(Length::Fill)
                            .align_x(Alignment::Start)
                            .padding([12.0, 16.0]),
                        // Language grid below
                        container(language_controls)
                            .width(Length::Fill)
                            .padding([0.0, 16.0]),
                    ]
                    .spacing(0)
                )
                .style(section_style)
            )
            .padding([16, 16]) // Extra top padding to show it's part of the provider section
            .width(Length::Fill)
            .into()
        } else {
            // No voices loaded - don't show anything
            column![].spacing(0).into()
        }
    } else {
        column![].spacing(0).into()
    };

    let provider_section = container(
        column![
            row![
                container(
                    white_text("Text-to-Speech Provider", 14)
                        .style(|_theme| iced::widget::text::Style {
                            color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.85)),
                        })
                )
                .width(Length::Fixed(120.0))
                .align_x(Alignment::Start),
                Space::new().width(Length::Fixed(16.0)),
                container(provider_controls)
                    .width(Length::Shrink)
                    .align_x(Alignment::Start),
            ]
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .padding([12.0, 16.0]),
            error_display,
            polly_error_display,
            piper_voice_section,
            polly_voice_section,
        ]
        .spacing(8)
    )
    .style(section_style);

    // Log Level section - compact horizontal layout
    let log_level_controls = row![
        radio("Error", LogLevel::Error, Some(app.log_level), Message::LogLevelSelected)
            .style(white_radio_style),
        radio("Warn", LogLevel::Warn, Some(app.log_level), Message::LogLevelSelected)
            .style(white_radio_style),
        radio("Info", LogLevel::Info, Some(app.log_level), Message::LogLevelSelected)
            .style(white_radio_style),
        radio("Debug", LogLevel::Debug, Some(app.log_level), Message::LogLevelSelected)
            .style(white_radio_style),
        radio("Trace", LogLevel::Trace, Some(app.log_level), Message::LogLevelSelected)
            .style(white_radio_style),
    ]
    .spacing(16);

    let log_level_section = container(
        row![
            container(
                white_text("Log Level", 14)
                    .style(|_theme| iced::widget::text::Style {
                        color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.85)),
                    })
            )
            .width(Length::Fixed(120.0))
            .align_x(Alignment::Start),
            Space::new().width(Length::Fixed(16.0)),
            container(log_level_controls)
                .width(Length::Fill)
                .align_x(Alignment::Start),
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .padding([12.0, 16.0])
    )
    .style(section_style);

    // Natural Reading section
    let text_cleanup_control = row![
        checkbox(app.text_cleanup_enabled)
            .label("Enable Natural Reading (cloud-powered text enhancement)")
            .on_toggle(Message::TextCleanupToggled)
            .style(white_checkbox_style),
        Space::new().width(Length::Fixed(8.0)),
        // Info icon button (circled i)
        button(
            container(
                white_text("ⓘ", 16)
                    .style(|_theme| iced::widget::text::Style {
                        color: Some(Color::from_rgb(0.3, 0.6, 1.0)),
                    })
            )
            .width(Length::Fixed(24.0))
            .height(Length::Fixed(24.0))
            .center_x(Length::Fixed(24.0))
            .center_y(Length::Fixed(24.0))
        )
        .style(transparent_button_style)
        .width(Length::Fixed(24.0))
        .height(Length::Fixed(24.0))
        .on_press(Message::OpenTextCleanupInfo),
    ]
    .align_y(Alignment::Center)
    .spacing(0);

    let text_cleanup_section = container(
        row![
            container(
                white_text("Natural Reading", 14)
                    .style(|_theme| iced::widget::text::Style {
                        color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.85)),
                    })
            )
            .width(Length::Fixed(120.0))
            .align_x(Alignment::Start),
            Space::new().width(Length::Fixed(16.0)),
            container(text_cleanup_control)
                .width(Length::Fill)
                .align_x(Alignment::Start),
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .padding([12.0, 16.0])
    )
    .style(section_style);

    // OCR section
    // Platform-specific OCR backend label
    let default_ocr_label = {
        #[cfg(target_os = "macos")]
        {
            "Standard OCR (Apple Vision Framework, local)"
        }
        #[cfg(target_os = "linux")]
        {
            "Standard OCR (EasyOCR, local)"
        }
        #[cfg(target_os = "windows")]
        {
            "Standard OCR (EasyOCR, local)"
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            "Standard OCR (local)"
        }
    };
    
    let ocr_controls = column![
        radio(
            default_ocr_label,
            OCRBackend::Default,
            Some(app.selected_ocr_backend),
            Message::OCRBackendSelected
        )
        .style(white_radio_style),
        Space::new().height(Length::Fixed(6.0)),
        row![
            radio(
                "Better OCR (Cloud service) - Coming soon",
                OCRBackend::BetterOCR,
                Some(app.selected_ocr_backend),
                Message::OCRBackendSelected
            )
            .style(|theme, status| {
                let mut style = white_radio_style(theme, status);
                // Make it appear disabled with reduced opacity
                style.text_color = Some(Color::from_rgba(1.0, 1.0, 1.0, 0.4));
                style.border_color = Color::from_rgba(1.0, 1.0, 1.0, 0.3);
                style.dot_color = Color::from_rgba(0.4, 0.6, 1.0, 0.4);
                style
            }),
            Space::new().width(Length::Fixed(8.0)),
            // Info icon button (circled i)
            button(
                container(
                    white_text("ⓘ", 16)
                        .style(|_theme| iced::widget::text::Style {
                            color: Some(Color::from_rgb(0.3, 0.6, 1.0)),
                        })
                )
                .width(Length::Fixed(24.0))
                .height(Length::Fixed(24.0))
                .center_x(Length::Fixed(24.0))
                .center_y(Length::Fixed(24.0))
            )
            .style(transparent_button_style)
            .width(Length::Fixed(24.0))
            .height(Length::Fixed(24.0))
            .on_press(Message::OpenOCRInfo),
        ]
        .align_y(Alignment::Center)
        .spacing(0),
    ]
    .spacing(0);

    let ocr_section = container(
        row![
            container(
                white_text("OCR", 14)
                    .style(|_theme| iced::widget::text::Style {
                        color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.85)),
                    })
            )
            .width(Length::Fixed(120.0))
            .align_x(Alignment::Start),
            Space::new().width(Length::Fixed(16.0)),
            container(ocr_controls)
                .width(Length::Shrink)
                .align_x(Alignment::Start),
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .padding([12.0, 16.0])
    )
    .style(section_style);

    container(
        column![
            modal_header("Settings", Message::CloseSettings),
            // Scrollable content area
            scrollable(
                container(
                    column![
                        ocr_section,
                        Space::new().height(Length::Fixed(12.0)),
                        text_cleanup_section,
                        Space::new().height(Length::Fixed(12.0)),
                        provider_section,
                        Space::new().height(Length::Fixed(12.0)),
                        log_level_section,
                    ]
                    .padding([20.0, 24.0])
                    .spacing(0)
                    .align_x(Alignment::Start),
                )
                .width(Length::Fill)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgb(0.12, 0.12, 0.14))),
                    ..Default::default()
                }),
            )
            .width(Length::Fill)
            .height(Length::Fill),
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill),
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
    let play_pause_icon: Element<Message> = if app.playback_state == PlaybackState::Playing {
        pause_icon(16.0).into()
    } else {
        play_icon(16.0).into()
    };

    // 3. Control buttons row
    let controls = row![
        circle_button(white_text("-5s", 12), Message::SkipBackward),
        circle_button(white_text("+5s", 12), Message::SkipForward),
        circle_button(play_pause_icon, Message::PlayPause),
        circle_button(stop_icon(16.0), Message::Stop),
        circle_button(camera_icon(16.0), Message::ScreenshotRequested),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    // 4. Base content row (without gear): [volume] [waveform] [controls]
    let content_row = row![
        volume_icon(28.0),
        Space::new().width(Length::Fixed(12.0)),
        waveform,
        Space::new().width(Length::Fixed(12.0)),
        controls,
    ]
    .align_y(Alignment::Center)
    .padding([8.0, 16.0]);

    // 5. Progress bar OR status text directly under the content row (not under gear)
    // Progress bar extends from left edge of content_row to right edge of screenshot button
    let (progress_or_status, gap_height): (Element<Message>, f32) = if let Some(status) = &app.status_text {
        // Show status text during loading (pushed up above where progress bar would be)
        let elem = container(
            text(status)
                .size(11)
                .style(|_theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
                }),
        )
        .width(Length::Fill)
        .height(Length::Fixed(33.0))
        .padding([-6.0, 16.0])
        .into();
        (elem, -8.0)
    } else {
        // Show progress bar during playback (stays in same position)
        // Extends from left padding (16.0) to end of screenshot button
        let elem = container(progress_bar(0.0..=1.0, app.progress))
            .width(Length::Fill)
            .height(Length::Fixed(1.0))
            .padding([0.0, 16.0])
            .into();
        (elem, 3.0)
    };

    let content_column = column![
        content_row,
        Space::new().height(Length::Fixed(gap_height)),
        progress_or_status,
    ]
    .width(Length::Shrink);

    // 6. Settings gear (transparent button) on the right
    let settings_btn = button(settings_icon(18.0))
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

    // 8. Outer container with window styling, wrapped in mouse_area for dragging
    mouse_area(
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(window_style),
    )
    .on_press(Message::StartDrag)
    .into()
}

/// Voice selection window view - shows voices for a selected language
pub fn voice_selection_window_view<'a>(app: &'a App) -> Element<'a, Message> {

    // Get voices for selected language (Piper or AWS)
    let voice_list: Element<'a, Message> = if let Some(ref lang_code) = app.selected_language.as_ref() {
        if app.selected_backend == TTSBackend::Piper {
            // Piper voices
            use crate::voices;
            if let Some(ref voices) = app.voices.as_ref() {
                let language_voices = voices::get_voices_for_language(voices, lang_code);
                
                if language_voices.is_empty() {
                    column![
                        white_text("No voices available for this language", 12)
                            .style(|_theme| iced::widget::text::Style {
                                color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.6)),
                            }),
                    ]
                    .spacing(0)
                    .into()
                } else {
                    let mut controls = column![].spacing(8);
                    
                    for voice in language_voices {
                        let voice_key = voice.key.clone();
                        let voice_name = format!("{} ({})", voice.name, voice.quality);
                        let is_selected = app.selected_voice.as_deref() == Some(&voice_key);
                        let is_downloaded = crate::voices::download::is_voice_downloaded(&voice_key);
                        let is_downloading = app.downloading_voice.as_deref() == Some(&voice_key);
                
                // Voice row: checkbox + name + quality + download/select button
                let voice_key_clone = voice_key.clone();
                let voice_row = if is_downloaded {
                    // Voice is downloaded - allow selection
                    row![
                        checkbox(is_selected)
                            .label(voice_name.clone())
                            .on_toggle(move |checked| {
                                if checked {
                                    Message::VoiceSelected(voice_key_clone.clone())
                                } else {
                                    Message::CloseVoiceSelection // Deselect
                                }
                            })
                            .style(white_checkbox_style),
                        Space::new().width(Length::Fixed(8.0)),
                        button(white_text("Select", 11))
                            .style(transparent_button_style)
                            .padding([4.0, 8.0])
                            .on_press(Message::VoiceSelected(voice_key.clone())),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(8)
                } else if is_downloading {
                    // Voice is currently downloading - show animated spinner
                    // Create animated spinner using rotating characters
                    let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                    let spinner_idx = ((app.loading_animation_time * 10.0) as usize) % spinner_chars.len();
                    let spinner_text = format!("{} Downloading...", spinner_chars[spinner_idx]);
                    
                    row![
                        checkbox(false)
                            .label(voice_name.clone())
                            .style(white_checkbox_style),
                        Space::new().width(Length::Fixed(8.0)),
                        // Spinner: animated
                        container(
                            text(spinner_text)
                                .size(11)
                                .style(|_theme| iced::widget::text::Style {
                                    color: Some(Color::from_rgba(0.3, 0.8, 1.0, 0.9)),
                                })
                        )
                        .width(Length::Fixed(120.0))
                        .align_x(Alignment::Center),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(8)
                } else {
                    // Voice not downloaded - disable checkbox, show download button
                    row![
                        checkbox(false)
                            .label(voice_name.clone())
                            .style(white_checkbox_style),
                        Space::new().width(Length::Fixed(8.0)),
                        button(white_text("Download", 11))
                            .style(transparent_button_style)
                            .padding([4.0, 8.0])
                            .on_press(Message::VoiceDownloadRequested(voice_key.clone())),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(8)
                };
                
                        controls = controls.push(voice_row);
                    }
                    
                    scrollable(controls).into()
                }
            } else {
                column![
                    white_text("Voices not loaded", 12)
                        .style(|_theme| iced::widget::text::Style {
                            color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.6)),
                        }),
                ]
                .spacing(0)
                .into()
            }
        } else if app.selected_backend == TTSBackend::AwsPolly {
            // AWS Polly voices - only show if voices are loaded
            use crate::voices::aws;
            
            if let Some(ref voices) = app.polly_voices.as_ref() {
                let language_voices = aws::get_voices_for_language(voices, lang_code);
                
                if language_voices.is_empty() {
                    column![
                        white_text("No voices available for this language", 12)
                            .style(|_theme| iced::widget::text::Style {
                                color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.6)),
                            }),
                    ]
                    .spacing(0)
                    .into()
                } else {
                    // Sort voices alphabetically by name, then by engine type
                    let mut sorted_voices: Vec<_> = language_voices.iter().collect();
                    sorted_voices.sort_by(|a, b| {
                        // First sort by voice name
                        let name_cmp = a.name.cmp(&b.name);
                        if name_cmp != std::cmp::Ordering::Equal {
                            return name_cmp;
                        }
                        // Then by engine type (Standard, Neural, Generative, LongForm)
                        let engine_order = |e: &str| match e {
                            "Standard" => 0,
                            "Neural" => 1,
                            "Generative" => 2,
                            "LongForm" => 3,
                            _ => 4,
                        };
                        engine_order(&a.engine).cmp(&engine_order(&b.engine))
                    });
                    
                    let mut controls = column![].spacing(8);
                    
                    for voice in sorted_voices {
                        // Use format "VoiceId:Engine" as the key to distinguish engine variants
                        let voice_key = format!("{}:{}", voice.id, voice.engine);
                        let engine_display = engine_display_name(&voice.engine);
                        let voice_name = format!("{} ({}, {})", voice.name, voice.gender, engine_display);
                        let is_selected = app.selected_polly_voice.as_deref() == Some(&voice_key);
                        
                        // AWS voices are always available (no download needed)
                        let voice_key_clone = voice_key.clone();
                        let voice_row = row![
                            checkbox(is_selected)
                                .label(voice_name.clone())
                                .on_toggle(move |checked| {
                                    if checked {
                                        Message::VoiceSelected(voice_key_clone.clone())
                                    } else {
                                        Message::CloseVoiceSelection // Deselect
                                    }
                                })
                                .style(white_checkbox_style),
                            Space::new().width(Length::Fixed(8.0)),
                            button(white_text("Select", 11))
                                .style(transparent_button_style)
                                .padding([4.0, 8.0])
                                .on_press(Message::VoiceSelected(voice_key.clone())),
                        ]
                        .align_y(Alignment::Center)
                        .spacing(8);
                        
                        controls = controls.push(voice_row);
                    }
                    
                    scrollable(controls).into()
                }
            } else {
                // No voices loaded - don't show anything
                column![].spacing(0).into()
            }
        } else {
            column![
                white_text("No backend selected", 12)
                    .style(|_theme| iced::widget::text::Style {
                        color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.6)),
                    }),
            ]
            .spacing(0)
            .into()
        }
    } else {
        column![
            white_text("No language selected", 12)
                .style(|_theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.6)),
                }),
        ]
        .spacing(0)
        .into()
    };

    // Get language info for header (outside the voice_list scope)
    let (header_flag_icon, language_name): (Element<'a, Message>, String) = if let Some(lang_code) = &app.selected_language {
        let flag_icon = flags::get_flag_icon(lang_code);
        
        let lang_info: Option<LanguageInfo> = match app.selected_backend {
            TTSBackend::Piper => app.voices.as_ref().and_then(|v| {
                use crate::voices;
                voices::get_available_languages(v)
                    .into_iter()
                    .find(|(code, _)| code == lang_code)
                    .map(|(_, info)| info)
            }),
            TTSBackend::AwsPolly => app.polly_voices.as_ref().and_then(|v| {
                v.values()
                    .find(|voice| voice.language.code == *lang_code)
                    .map(|voice| voice.language.clone())
            }),
        };
        
        let name = if let Some(lang_info) = lang_info {
            format!("{} ({})", lang_info.name_english, lang_code)
        } else {
            lang_code.to_string()
        };
        (flag_icon.into(), name)
    } else {
        // Fallback: globe icon for unknown language
        let globe_icon = flags::get_flag_icon("unknown");
        (globe_icon.into(), "Unknown Language".to_string())
    };

    container(
        column![
            container(
                row![
                    text("Select voice in ").size(18)
                        .style(|_theme| iced::widget::text::Style {
                            color: Some(Color::WHITE),
                        }),
                    header_flag_icon,
                    Space::new().width(Length::Fixed(6.0)),
                    text(language_name).size(18)
                        .style(|_theme| iced::widget::text::Style {
                            color: Some(Color::WHITE),
                        }),
                    Space::new().width(Length::Fill),
                    close_button(Message::CloseVoiceSelection),
                ]
                .width(Length::Fill)
                .align_y(Alignment::Center)
            )
            .width(Length::Fill)
            .padding([20.0, 24.0])
            .style(header_style),
            // Scrollable voice list
            scrollable(
                container(
                    column![
                        container(voice_list)
                            .width(Length::Fill)
                            .padding([20.0, 24.0]),
                    ]
                    .spacing(0)
                )
                .width(Length::Fill)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgb(0.12, 0.12, 0.14))),
                    ..Default::default()
                }),
            )
            .width(Length::Fill)
            .height(Length::Fill),
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(modal_content_style)
    .into()
}

/// AWS Polly pricing information modal window
pub fn polly_info_window_view<'a>(_app: &'a App) -> Element<'a, Message> {
    container(
        column![
            modal_header("AWS Polly Pricing Information", Message::ClosePollyInfo),
            // Content area
            scrollable(
                container(
                    column![
                        container(
                            white_text("Important: Please check AWS pricing", 16)
                                .style(|_theme| iced::widget::text::Style {
                                    color: Some(Color::WHITE),
                                })
                        )
                        .width(Length::Fill)
                        .padding([20.0, 24.0]),
                        container(
                            white_text(
                                "AWS Polly charges based on the number of characters processed. \
                                Standard voices cost $4.00 per 1 million characters, Neural voices cost $16.00 per 1 million characters, \
                                and Long-Form voices cost $100.00 per 1 million characters. \
                                Generative voices cost $30.00 per 1 million characters.\n\n\
                                Free tier includes:\n\
                                • Standard voices: 5 million characters per month\n\
                                • Neural voices: 1 million characters per month (first 12 months)\n\
                                • Long-Form voices: 500 thousand characters per month (first 12 months)\n\
                                • Generative voices: 100 thousand characters per month (first 12 months)\n\n\
                                Please review AWS pricing before using this service to understand potential charges.",
                                13
                            )
                            .style(|_theme| iced::widget::text::Style {
                                color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.85)),
                            })
                        )
                        .width(Length::Fill)
                        .padding([0.0, 24.0]),
                        Space::new().height(Length::Fixed(16.0)),
                        container(
                            button(
                                white_text("View AWS Polly Pricing Details →", 13)
                                    .style(|_theme| iced::widget::text::Style {
                                        color: Some(Color::from_rgb(0.3, 0.6, 1.0)),
                                    })
                            )
                            .style(transparent_button_style)
                            .padding([8.0, 12.0])
                            .on_press(Message::OpenPollyPricingUrl)
                        )
                        .width(Length::Fill)
                        .padding([0.0, 24.0])
                        .align_x(Alignment::Start),
                        Space::new().height(Length::Fixed(20.0)),
                    ]
                    .spacing(12)
                )
                .width(Length::Fill)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgb(0.12, 0.12, 0.14))),
                    ..Default::default()
                }),
            )
            .width(Length::Fill)
            .height(Length::Fill),
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(modal_content_style)
    .into()
}

/// Natural Reading information modal window
pub fn text_cleanup_info_window_view<'a>(_app: &'a App) -> Element<'a, Message> {
    container(
        column![
            modal_header("Natural Reading", Message::CloseTextCleanupInfo),
            // Content area
            scrollable(
                container(
                    column![
                        container(
                            white_text("Natural Reading", 16)
                                .style(|_theme| iced::widget::text::Style {
                                    color: Some(Color::WHITE),
                                })
                        )
                        .width(Length::Fill)
                        .padding([20.0, 24.0]),
                        container(
                            white_text(
                    "Transform raw text into polished, natural-sounding speech with our cloud-powered text enhancement service.

Natural Reading intelligently enhances your text by:
• Removing noise and formatting artifacts
• Improving punctuation and sentence structure
• Optimizing content for natural speech patterns
• Preserving context and meaning

Perfect for reading websites, chat conversations (Slack, Discord, etc.), structured content like tables, and any text that needs refinement before text-to-speech conversion.",
                                13
                            )
                            .style(|_theme| iced::widget::text::Style {
                                color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.85)),
                            })
                        )
                        .width(Length::Fill)
                        .padding([0.0, 24.0]),
                        Space::new().height(Length::Fixed(20.0)),
                    ]
                    .spacing(12)
                )
                .width(Length::Fill)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgb(0.12, 0.12, 0.14))),
                    ..Default::default()
                }),
            )
            .width(Length::Fill)
            .height(Length::Fill),
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(modal_content_style)
    .into()
}

/// Better OCR information modal window
pub fn ocr_info_window_view<'a>(_app: &'a App) -> Element<'a, Message> {
    container(
        column![
            modal_header("Better OCR Information", Message::CloseOCRInfo),
            // Content area
            scrollable(
                container(
                    column![
                        container(
                            white_text("Better OCR - Cloud Service", 16)
                                .style(|_theme| iced::widget::text::Style {
                                    color: Some(Color::WHITE),
                                })
                        )
                        .width(Length::Fill)
                        .padding([20.0, 24.0]),
                        container(
                            white_text(
                                "Better OCR is a cloud service offered by Insight Reader that combines OCR and AI to nearly eliminate errors in screenshots and deliver exceptional accuracy on scanned documents and handwritten text.",
                                13
                            )
                            .style(|_theme| iced::widget::text::Style {
                                color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.85)),
                            })
                        )
                        .width(Length::Fill)
                        .padding([0.0, 24.0]),
                        Space::new().height(Length::Fixed(20.0)),
                    ]
                    .spacing(12)
                )
                .width(Length::Fill)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgb(0.12, 0.12, 0.14))),
                    ..Default::default()
                }),
            )
            .width(Length::Fill)
            .height(Length::Fill),
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(modal_content_style)
    .into()
}

/// Extracted text dialog window - displays extracted text with copy button
pub fn extracted_text_dialog_view<'a>(app: &'a App) -> Element<'a, Message> {

    // Display the extracted text in an editable text area
    let text_content: Element<'a, Message> = if let Some(ref editor_content) = app.extracted_text_editor {
        // Use text_editor widget for multi-line editing
        container(
            text_editor(editor_content)
                .on_action(Message::ExtractedTextEditorAction)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(8)
        .into()
    } else if app.extracted_text.is_some() {
        // Fallback: show message if editor not initialized
        container(
            white_text("Initializing editor...", 14)
                .style(|_theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.6)),
                })
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    } else {
        container(
            white_text("No text available", 14)
                .style(|_theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.6)),
                })
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    };

    // Copy button - with SVG icon and text
    let copy_button = button(
        container(
            row![
                clipboard_icon(16.0),
                Space::new().width(Length::Fixed(6.0)),
                white_text("Copy to Clipboard", 13)
                    .style(|_theme| iced::widget::text::Style {
                        color: Some(Color::WHITE),
                    })
            ]
            .align_y(Alignment::Center)
        )
        .padding([8.0, 16.0])
    )
    .style(transparent_button_style)
    .on_press(Message::CopyExtractedTextToClipboard);

    // Read button - with SVG icon and text
    let read_button = button(
        container(
            row![
                play_icon(16.0),
                Space::new().width(Length::Fixed(6.0)),
                white_text("Read", 13)
                    .style(|_theme| iced::widget::text::Style {
                        color: Some(Color::WHITE),
                    })
            ]
            .align_y(Alignment::Center)
        )
        .padding([8.0, 16.0])
    )
    .style(transparent_button_style)
    .on_press(Message::ReadExtractedText);

    container(
        column![
            container(
                row![
                    white_text("Extracted Text", 20)
                        .style(|_theme| iced::widget::text::Style {
                            color: Some(Color::WHITE),
                        }),
                    Space::new().width(Length::Fill),
                    read_button,
                    Space::new().width(Length::Fixed(4.0)),
                    copy_button,
                    Space::new().width(Length::Fixed(16.0)),
                    close_button(Message::CloseExtractedTextDialog),
                ]
                .width(Length::Fill)
                .align_y(Alignment::Center)
            )
            .width(Length::Fill)
            .padding([20.0, 24.0])
            .style(header_style),
            // Text content area (editable text input)
            text_content,
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(modal_content_style)
    .into()
}

/// Screenshot viewer window - displays the captured screenshot
pub fn screenshot_viewer_view<'a>(app: &'a App) -> Element<'a, Message> {

    // Display the screenshot image if available
    let image_content: Element<'a, Message> = if let Some(ref screenshot_path) = app.screenshot_path {
        // Load image from file path using Iced's image widget
        use iced::widget::image::{Image, Handle};
        let image_handle = Handle::from_path(screenshot_path);
        let img = Image::new(image_handle)
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(ContentFit::Contain);
        
        container(img)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    } else {
        container(
            white_text("No screenshot available", 14)
                .style(|_theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.6)),
                })
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    };

    container(
        column![
            modal_header("Screenshot", Message::CloseScreenshotViewer),
            // Image content area
            container(image_content)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgb(0.12, 0.12, 0.14))),
                    ..Default::default()
                }),
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(modal_content_style)
    .into()
}
