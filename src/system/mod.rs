//! System interactions (clipboard, external commands, etc.)

mod clipboard;
mod text_cleanup;
mod screenshot;

pub use clipboard::get_selected_text;
pub use text_cleanup::cleanup_text;
pub use screenshot::{capture_region, extract_text_from_image};


