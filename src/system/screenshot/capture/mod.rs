//! Screenshot region capture functionality

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

/// Captures a screenshot of a selected screen region.
/// 
/// On macOS, uses `screencapture -i` for interactive region selection.
/// On Linux, tries multiple screenshot tools in order of preference.
/// Returns the path to the captured image file, or an error message.
pub fn capture_region() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        macos::capture_region_macos()
    }
    
    #[cfg(target_os = "linux")]
    {
        linux::capture_region_linux()
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        tracing::warn!("Screenshot region selection not supported on this platform");
        Err("Screenshot region selection is only supported on macOS and Linux".to_string())
    }
}
