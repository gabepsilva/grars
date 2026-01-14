//! System interactions (clipboard, external commands, etc.)

mod clipboard;
mod text_cleanup;
mod screenshot;
mod tray;

pub use clipboard::{get_selected_text, copy_to_clipboard};
pub use text_cleanup::cleanup_text;
pub use screenshot::{capture_region, extract_text_from_image};
pub use tray::{SystemTray, TrayEvent};


