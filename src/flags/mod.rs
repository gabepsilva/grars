//! Flag icons module - SVG flag icons for language selection UI.
//!
//! Provides SVG flag icons that render consistently across all platforms,
//! replacing emoji flags which don't display correctly on Windows.

mod lang_mapping;
mod svg_data;

pub use lang_mapping::lang_to_country;
pub use svg_data::get_flag_svg;

use iced::widget::svg;
use iced::Length;

/// Get flag SVG widget for a language code (e.g., "en_US", "pt_BR").
///
/// Returns a 20x15 pixel SVG flag icon. Falls back to a globe icon
/// for unknown country codes.
pub fn get_flag_icon(lang_code: &str) -> svg::Svg<'static> {
    let country = lang_to_country(lang_code);
    let bytes = get_flag_svg(country);
    svg(svg::Handle::from_memory(bytes))
        .width(Length::Fixed(20.0))
        .height(Length::Fixed(15.0))
}
