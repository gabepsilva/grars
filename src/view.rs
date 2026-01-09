//! UI rendering logic

use iced::widget::{button, checkbox, column, container, mouse_area, progress_bar, radio, row, scrollable, svg, text, Space};
use iced::{Alignment, Background, Color, Element, Length};

use crate::model::{App, LogLevel, Message, PlaybackState, TTSBackend};
use crate::styles::{
    circle_button_style, close_button_style, error_container_style, header_style,
    modal_content_style, section_style, transparent_button_style, wave_bar_style,
    white_checkbox_style, white_radio_style, window_style,
};

const MIN_HEIGHT: f32 = 4.0;
const MAX_HEIGHT: f32 = 24.0;
const NUM_BARS: usize = 10;

/// Get flag emoji for a language code
fn get_flag_emoji(lang_code: &str) -> &'static str {
    // Extract country code from language code (e.g., "pt_BR" -> "BR")
    let country = lang_code.split('_').nth(1).unwrap_or("");
    
    match country {
        // Portuguese variants
        "BR" => "🇧🇷",
        "PT" => "🇵🇹",
        // English variants
        "US" => "🇺🇸",
        "GB" => "🇬🇧",
        "AU" => "🇦🇺",
        "CA" => "🇨🇦",
        // Spanish variants
        "ES" => "🇪🇸",
        "MX" => "🇲🇽",
        "AR" => "🇦🇷",
        "CO" => "🇨🇴",
        // French variants
        "FR" => "🇫🇷",
        // German variants
        "DE" => "🇩🇪",
        "AT" => "🇦🇹",
        "CH" => "🇨🇭",
        // Other European
        "IT" => "🇮🇹",
        "NL" => "🇳🇱",
        "PL" => "🇵🇱",
        "RU" => "🇷🇺",
        "TR" => "🇹🇷",
        "GR" => "🇬🇷",
        "CZ" => "🇨🇿",
        "SK" => "🇸🇰",
        "HU" => "🇭🇺",
        "RO" => "🇷🇴",
        "BG" => "🇧🇬",
        "HR" => "🇭🇷",
        "SI" => "🇸🇮",
        "FI" => "🇫🇮",
        "SV" => "🇸🇪",
        "NO" => "🇳🇴",
        "DA" => "🇩🇰",
        "IS" => "🇮🇸",
        "EE" => "🇪🇪",
        "LV" => "🇱🇻",
        "LT" => "🇱🇹",
        // Asian
        "CN" => "🇨🇳",
        "TW" => "🇹🇼",
        "HK" => "🇭🇰",
        "JP" => "🇯🇵",
        "KR" => "🇰🇷",
        "VN" => "🇻🇳",
        "TH" => "🇹🇭",
        "ID" => "🇮🇩",
        "MY" => "🇲🇾",
        "PH" => "🇵🇭",
        "IN" => "🇮🇳",
        "PK" => "🇵🇰",
        "BD" => "🇧🇩",
        // Middle East
        "SA" => "🇸🇦",
        "AE" => "🇦🇪",
        "IL" => "🇮🇱",
        "IR" => "🇮🇷",
        "IQ" => "🇮🇶",
        "JO" => "🇯🇴",
        // African
        "ZA" => "🇿🇦",
        "EG" => "🇪🇬",
        "KE" => "🇰🇪",
        "NG" => "🇳🇬",
        // Americas
        "CL" => "🇨🇱",
        "PE" => "🇵🇪",
        "VE" => "🇻🇪",
        "EC" => "🇪🇨",
        "BO" => "🇧🇴",
        "PY" => "🇵🇾",
        "UY" => "🇺🇾",
        "CR" => "🇨🇷",
        "PA" => "🇵🇦",
        "DO" => "🇩🇴",
        "CU" => "🇨🇺",
        // Fallback: use language family
        _ => {
            let lang_family = lang_code.split('_').next().unwrap_or("");
            match lang_family {
                "ar" => "🇸🇦", // Arabic
                "zh" => "🇨🇳", // Chinese
                "hi" => "🇮🇳", // Hindi
                "ja" => "🇯🇵", // Japanese
                "ko" => "🇰🇷", // Korean
                "th" => "🇹🇭", // Thai
                "vi" => "🇻🇳", // Vietnamese
                "cs" => "🇨🇿", // Czech
                "sk" => "🇸🇰", // Slovak
                "hu" => "🇭🇺", // Hungarian
                "ro" => "🇷🇴", // Romanian
                "bg" => "🇧🇬", // Bulgarian
                "hr" => "🇭🇷", // Croatian
                "sr" => "🇷🇸", // Serbian
                "sl" => "🇸🇮", // Slovenian
                "et" => "🇪🇪", // Estonian
                "lv" => "🇱🇻", // Latvian
                "lt" => "🇱🇹", // Lithuanian
                "fi" => "🇫🇮", // Finnish
                "sv" => "🇸🇪", // Swedish
                "no" => "🇳🇴", // Norwegian
                "da" => "🇩🇰", // Danish
                "is" => "🇮🇸", // Icelandic
                "ca" => "🇪🇸", // Catalan
                "eu" => "🇪🇸", // Basque
                "gl" => "🇪🇸", // Galician
                "uk" => "🇺🇦", // Ukrainian
                "be" => "🇧🇾", // Belarusian
                "mk" => "🇲🇰", // Macedonian
                "sq" => "🇦🇱", // Albanian
                "mt" => "🇲🇹", // Maltese
                "ga" => "🇮🇪", // Irish
                "cy" => "🇬🇧", // Welsh
                "he" => "🇮🇱", // Hebrew
                "fa" => "🇮🇷", // Persian
                "ur" => "🇵🇰", // Urdu
                "bn" => "🇧🇩", // Bengali
                "ta" => "🇮🇳", // Tamil
                "te" => "🇮🇳", // Telugu
                "ml" => "🇮🇳", // Malayalam
                "kn" => "🇮🇳", // Kannada
                "gu" => "🇮🇳", // Gujarati
                "pa" => "🇮🇳", // Punjabi
                "mr" => "🇮🇳", // Marathi
                "ne" => "🇳🇵", // Nepali
                "si" => "🇱🇰", // Sinhala
                "my" => "🇲🇲", // Burmese
                "km" => "🇰🇭", // Khmer
                "lo" => "🇱🇦", // Lao
                "ka" => "🇬🇪", // Georgian
                "hy" => "🇦🇲", // Armenian
                "az" => "🇦🇿", // Azerbaijani
                "kk" => "🇰🇿", // Kazakh
                "ky" => "🇰🇬", // Kyrgyz
                "uz" => "🇺🇿", // Uzbek
                "mn" => "🇲🇳", // Mongolian
                "sw" => "🇰🇪", // Swahili
                "af" => "🇿🇦", // Afrikaans
                "am" => "🇪🇹", // Amharic
                "yo" => "🇳🇬", // Yoruba
                "ig" => "🇳🇬", // Igbo
                "ha" => "🇳🇬", // Hausa
                "zu" => "🇿🇦", // Zulu
                "xh" => "🇿🇦", // Xhosa
                "st" => "🇿🇦", // Southern Sotho
                "tn" => "🇿🇦", // Tswana
                "sn" => "🇿🇼", // Shona
                "ny" => "🇲🇼", // Chichewa
                "so" => "🇸🇴", // Somali
                "om" => "🇪🇹", // Oromo
                "ti" => "🇪🇷", // Tigrinya
                "mg" => "🇲🇬", // Malagasy
                "rw" => "🇷🇼", // Kinyarwanda
                "lg" => "🇺🇬", // Ganda
                "ak" => "🇬🇭", // Akan
                "ff" => "🇸🇳", // Fulah
                "wo" => "🇸🇳", // Wolof
                "bm" => "🇲🇱", // Bambara
                "ee" => "🇬🇭", // Ewe
                "tw" => "🇬🇭", // Twi
                _ => "🌐", // Default globe emoji
            }
        }
    }
}

// Bundled SVG icons (embedded at compile time)
const SVG_PLAY: &[u8] = include_bytes!("../assets/icons/play.svg");
const SVG_PAUSE: &[u8] = include_bytes!("../assets/icons/pause.svg");
const SVG_STOP: &[u8] = include_bytes!("../assets/icons/stop.svg");
const SVG_VOLUME: &[u8] = include_bytes!("../assets/icons/volume.svg");
const SVG_SETTINGS: &[u8] = include_bytes!("../assets/icons/settings.svg");

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

/// Settings window view - floating modal style
pub fn settings_window_view<'a>(app: &'a App) -> Element<'a, Message> {
    let close_button = button(
        container(white_text("✕", 18))
            .width(Length::Fixed(28.0))
            .height(Length::Fixed(28.0))
            .center_x(Length::Fixed(28.0))
            .center_y(Length::Fixed(28.0)),
    )
    .style(close_button_style)
    .on_press(Message::CloseSettings);

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
        radio(
            "AWS Polly (Cloud, BYO credentials)",
            TTSBackend::AwsPolly,
            Some(app.selected_backend),
            Message::ProviderSelected
        )
        .style(white_radio_style),
    ]
    .spacing(0);

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
                let flag_emoji = get_flag_emoji(lang_code);
                let label = format!("{} {} ({})", flag_emoji, lang_info.name_english, lang_code);
                let lang_code_clone = lang_code.clone();
                let is_selected = app.selected_language.as_ref().map(|s| s == lang_code).unwrap_or(false);
                
                // Create button with owned string - clicking opens voice selection immediately
                let lang_button = button(
                    container(
                        text(label).size(13)
                            .style(move |_theme| iced::widget::text::Style {
                                color: Some(if is_selected {
                                    Color::WHITE
                                } else {
                                    Color::from_rgba(1.0, 1.0, 1.0, 0.7)
                                }),
                            })
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

    let provider_section = container(
        column![
            row![
                container(
                    white_text("TTS Provider", 14)
                        .style(|_theme| iced::widget::text::Style {
                            color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.85)),
                        })
                )
                .width(Length::Fixed(120.0))
                .align_x(Alignment::Start),
                Space::new().width(Length::Fixed(16.0)),
                container(provider_controls)
                    .width(Length::Fill)
                    .align_x(Alignment::Start),
            ]
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .padding([12.0, 16.0]),
            error_display,
            piper_voice_section,
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

    // Text Cleanup section
    let text_cleanup_control = checkbox(app.text_cleanup_enabled)
        .label("Enable text cleanup (sends text to local API before TTS)")
        .on_toggle(Message::TextCleanupToggled)
        .style(white_checkbox_style);

    let text_cleanup_section = container(
        row![
            container(
                white_text("Text Cleanup", 14)
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

    container(
        column![
            // Header bar at top (fixed, not scrollable)
            container(
                row![
                    white_text("Settings", 20)
                        .style(|_theme| iced::widget::text::Style {
                            color: Some(Color::WHITE),
                        }),
                    Space::new().width(Length::Fill),
                    close_button,
                ]
                .width(Length::Fill)
                .align_y(Alignment::Center)
            )
            .width(Length::Fill)
            .padding([20.0, 24.0])
            .style(header_style),
            // Scrollable content area
            scrollable(
                container(
                    column![
                        provider_section,
                        Space::new().height(Length::Fixed(12.0)),
                        log_level_section,
                        Space::new().height(Length::Fixed(12.0)),
                        text_cleanup_section,
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
    let (progress_or_status, gap_height): (Element<Message>, f32) = if let Some(status) = &app.status_text {
        // Show status text during loading (pushed up above where progress bar would be)
        let elem = container(
            text(status)
                .size(11)
                .style(|_theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
                }),
        )
        .width(Length::Fixed(313.0))
        .height(Length::Fixed(33.0))
        .padding([-6.0, 19.0])
        .into();
        (elem, -8.0)
    } else {
        // Show progress bar during playback (stays in same position)
        let elem = container(progress_bar(0.0..=1.0, app.progress))
            .width(Length::Fixed(313.0))
            .height(Length::Fixed(1.0))
            .padding([0.0, 19.0])
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
    let close_button = button(
        container(white_text("✕", 18))
            .width(Length::Fixed(28.0))
            .height(Length::Fixed(28.0))
            .center_x(Length::Fixed(28.0))
            .center_y(Length::Fixed(28.0)),
    )
    .style(close_button_style)
    .on_press(Message::CloseVoiceSelection);

    use crate::voices;
    
    // Get voices for selected language
    let voice_list: Element<'a, Message> = if let (Some(ref voices), Some(ref lang_code)) = (app.voices.as_ref(), app.selected_language.as_ref()) {
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
                let is_selected = app.selected_voice.as_ref().map(|s| s.as_str() == voice_key.as_str()).unwrap_or(false);
                let is_downloaded = crate::voices::download::is_voice_downloaded(&voice_key);
                let is_downloading = app.downloading_voice.as_ref().map(|s| s == &voice_key).unwrap_or(false);
                
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
            white_text("No language selected", 12)
                .style(|_theme| iced::widget::text::Style {
                    color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.6)),
                }),
        ]
        .spacing(0)
        .into()
    };

    // Get language name for header (outside the voice_list scope)
    let language_name: String = if let (Some(ref voices), Some(ref lang_code)) = (app.voices.as_ref(), app.selected_language.as_ref()) {
        let flag_emoji = get_flag_emoji(lang_code);
        if let Some((_, lang_info)) = voices::get_available_languages(voices)
            .iter()
            .find(|(code, _)| code.as_str() == lang_code.as_str())
        {
            format!("{} {} ({})", flag_emoji, lang_info.name_english, lang_code)
        } else {
            format!("{} {}", flag_emoji, lang_code)
        }
    } else {
        "Unknown Language".to_string()
    };

    container(
        column![
            // Header bar
            container(
                row![
                    text(format!("Select voice in {}", language_name))
                        .size(18)
                        .style(|_theme| iced::widget::text::Style {
                            color: Some(Color::WHITE),
                        }),
                    Space::new().width(Length::Fill),
                    close_button,
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
