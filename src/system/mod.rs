//! System interactions (clipboard, external commands, etc.)

mod clipboard;
mod text_cleanup;

pub use clipboard::get_selected_text;
pub use text_cleanup::cleanup_text;


