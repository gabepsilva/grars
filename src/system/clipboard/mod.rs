//! Clipboard and selection reading utilities

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(test)]
#[cfg(any(target_os = "macos", target_os = "linux"))]
mod tests;

use tracing::{debug, info, warn};

/// Creates a preview string for logging (first 200 chars).
pub(crate) fn text_preview(text: &str) -> String {
    let mut chars = text.chars();
    let preview: String = chars.by_ref().take(200).collect();
    if chars.next().is_some() {
        format!("{}...", preview)
    } else {
        preview
    }
}

/// Helper to process and return trimmed text if non-empty.
pub(crate) fn process_text(text: String, source: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        debug!("{} is empty", source);
        None
    } else {
        info!(bytes = trimmed.len(), "Successfully retrieved text from {}", source);
        debug!(text = %text_preview(trimmed), "Captured text content");
        Some(trimmed.to_string())
    }
}

/// Gets the currently selected text.
/// - On Linux: Uses arboard to read from PRIMARY selection first, falls back to clipboard
/// - On macOS: Uses arboard to read from clipboard
/// - On other platforms: Returns None
pub fn get_selected_text() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        macos::get_selected_text_macos()
    }
    
    #[cfg(target_os = "linux")]
    {
        linux::get_selected_text_linux()
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        warn!("Platform not supported for text selection");
        None
    }
}

/// Copies text to the clipboard.
/// - On macOS: Uses arboard
/// - On Linux: Uses arboard
/// - On other platforms: Returns an error
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        use arboard::Clipboard;
        
        let mut clipboard = Clipboard::new().map_err(|e| {
            warn!(error = %e, "Failed to initialize clipboard");
            format!("Failed to initialize clipboard: {}", e)
        })?;
        
        clipboard.set_text(text).map_err(|e| {
            warn!(error = %e, "Failed to copy to clipboard");
            format!("Failed to copy to clipboard: {}", e)
        })?;
        
        info!(bytes = text.len(), "Successfully copied text to clipboard");
        Ok(())
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        warn!("Platform not supported for clipboard copy");
        Err("Clipboard copy not supported on this platform".to_string())
    }
}
