//! System interactions (clipboard, external commands, etc.)

mod clipboard;
mod text_cleanup;
mod screenshot;
mod tray;
mod hotkey;

pub use clipboard::{get_selected_text, copy_to_clipboard};
pub use text_cleanup::cleanup_text;
pub use screenshot::{capture_region, extract_text_from_image};
pub use tray::{SystemTray, TrayEvent};
pub use hotkey::{HotkeyManager, HotkeyConfig, format_hotkey_display};

/// Check if running on Wayland with Hyprland compositor
#[cfg(target_os = "linux")]
pub fn is_wayland_hyprland() -> bool {
    // Check if we're on Wayland
    let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok()
        || std::env::var("XDG_SESSION_TYPE")
            .map(|s| s.to_lowercase() == "wayland")
            .unwrap_or(false);
    
    if !is_wayland {
        return false;
    }
    
    // Check if we're on Hyprland
    // Hyprland sets HYPRLAND_INSTANCE_SIGNATURE
    std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok()
        || std::env::var("XDG_CURRENT_DESKTOP")
            .map(|s| s.to_lowercase().contains("hyprland"))
            .unwrap_or(false)
}

/// Check if running on Wayland with Hyprland compositor
#[cfg(not(target_os = "linux"))]
pub fn is_wayland_hyprland() -> bool {
    false
}


