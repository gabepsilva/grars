//! Clipboard and selection reading utilities

use tracing::{debug, info, warn};

/// Creates a preview string for logging (first 200 chars).
pub(crate) fn text_preview(text: &str) -> String {
    if text.chars().count() > 200 {
        format!("{}...", text.chars().take(200).collect::<String>())
    } else {
        text.to_string()
    }
}

/// Gets the currently selected text.
/// - On Linux: Uses arboard to read from PRIMARY selection first, falls back to clipboard
/// - On macOS: Uses arboard to read from clipboard
/// - On other platforms: Returns None
pub fn get_selected_text() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        get_selected_text_macos()
    }
    
    #[cfg(target_os = "linux")]
    {
        get_selected_text_linux()
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        warn!("Platform not supported for text selection");
        None
    }
}

/// Helper to process and return trimmed text if non-empty.
fn process_text(text: String, source: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        debug!("{} is empty", source);
        None
    } else {
        info!(bytes = trimmed.len(), "Successfully retrieved text from {}", source);
        debug!(text = %text_preview(trimmed), "Captured text content");
        Some(trimmed.to_owned())
    }
}

#[cfg(target_os = "linux")]
fn get_selected_text_linux() -> Option<String> {
    use arboard::{Clipboard, GetExtLinux, LinuxClipboardKind};
    
    info!("Attempting to read selected text (PRIMARY selection, fallback to clipboard)");
    
    let mut clipboard = Clipboard::new().ok()?;
    
    // First attempt: Try PRIMARY selection (selected text)
    if let Ok(text) = clipboard.get().clipboard(LinuxClipboardKind::Primary).text() {
        if let Some(result) = process_text(text, "PRIMARY selection") {
            return Some(result);
        }
        debug!("PRIMARY selection is empty, falling back to clipboard");
    } else {
        debug!("PRIMARY selection unavailable, falling back to clipboard");
    }
    
    // Fallback: Try regular clipboard
    clipboard.get_text()
        .ok()
        .and_then(|text| process_text(text, "clipboard (fallback)"))
}

#[cfg(target_os = "macos")]
fn get_selected_text_macos() -> Option<String> {
    use arboard::Clipboard;
    
    Clipboard::new()
        .ok()?
        .get_text()
        .ok()
        .and_then(|text| process_text(text, "clipboard"))
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

#[cfg(test)]
#[cfg(any(target_os = "macos", target_os = "linux"))]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Serialize clipboard tests to avoid interference.
    // Tests that modify the clipboard should run sequentially.
    // For best results, run tests with: cargo test clipboard -- --test-threads=1
    static CLIPBOARD_MUTEX: Mutex<()> = Mutex::new(());

    // Helper to acquire clipboard mutex (handles poison recovery)
    fn clipboard_lock() -> std::sync::MutexGuard<'static, ()> {
        CLIPBOARD_MUTEX.lock().unwrap_or_else(|e| e.into_inner())
    }

    // Helper to wait for clipboard operations to complete
    const CLIPBOARD_DELAY_MS: u64 = 100;
    fn wait_for_clipboard() {
        std::thread::sleep(std::time::Duration::from_millis(CLIPBOARD_DELAY_MS));
    }

    // ============================================================================
    // Unit Tests for text_preview()
    // ============================================================================

    #[test]
    fn test_text_preview_short_string() {
        let text = "Hello, world!";
        let result = text_preview(text);
        assert_eq!(result, text);
    }

    #[test]
    fn test_text_preview_exactly_200_chars() {
        let text = "a".repeat(200);
        let result = text_preview(&text);
        assert_eq!(result, text);
        assert_eq!(result.len(), 200);
    }

    #[test]
    fn test_text_preview_long_string() {
        let text = "a".repeat(500);
        let result = text_preview(&text);
        assert!(result.len() == 203); // 200 chars + "..."
        assert!(result.ends_with("..."));
        assert_eq!(&result[..200], "a".repeat(200));
    }

    #[test]
    fn test_text_preview_empty_string() {
        let text = "";
        let result = text_preview(text);
        assert_eq!(result, "");
    }

    #[test]
    fn test_text_preview_unicode_emoji() {
        let text = "Hello 👋 World 🌍";
        let result = text_preview(text);
        assert_eq!(result, text);
    }

    #[test]
    fn test_text_preview_unicode_multibyte() {
        let text = "Привет мир"; // Russian text
        let result = text_preview(text);
        assert_eq!(result, text);
    }

    #[test]
    fn test_text_preview_unicode_long() {
        // Create a string with 250 emoji characters
        let text = "👋".repeat(250);
        let result = text_preview(&text);
        // Should truncate at 200 characters (which is 200 emojis)
        // Each emoji is 4 bytes, so 200 emojis = 800 bytes, but we count chars
        let char_count = result.chars().count();
        assert!(char_count <= 203, "Should truncate to 200 chars + '...', got {} chars", char_count);
        assert!(result.ends_with("..."));
        // Verify the first 200 characters are emojis
        let without_suffix = result.strip_suffix("...").unwrap();
        assert_eq!(without_suffix.chars().count(), 200);
    }

    // ============================================================================
    // Integration Tests for copy_to_clipboard()
    // ============================================================================

    #[test]
    fn test_copy_to_clipboard_simple_ascii() {
        let _guard = clipboard_lock();
        let text = "Hello, world!";
        let result = copy_to_clipboard(text);
        assert!(result.is_ok(), "Failed to copy simple ASCII text: {:?}", result);
    }

    #[test]
    fn test_copy_to_clipboard_empty_string() {
        let _guard = clipboard_lock();
        let text = "";
        let result = copy_to_clipboard(text);
        // Empty string copy should succeed (though clipboard may be empty)
        assert!(result.is_ok(), "Failed to copy empty string: {:?}", result);
    }

    #[test]
    fn test_copy_to_clipboard_long_text() {
        let _guard = clipboard_lock();
        let text = "a".repeat(5000);
        let result = copy_to_clipboard(&text);
        assert!(result.is_ok(), "Failed to copy long text: {:?}", result);
    }

    #[test]
    fn test_copy_to_clipboard_special_characters() {
        let _guard = clipboard_lock();
        let text = "Line 1\nLine 2\tTabbed\tText\n\"Quoted\" 'Single'";
        let result = copy_to_clipboard(text);
        assert!(result.is_ok(), "Failed to copy text with special characters: {:?}", result);
    }

    #[test]
    fn test_copy_to_clipboard_unicode() {
        let _guard = clipboard_lock();
        let text = "Hello 👋 World 🌍 Привет мир";
        let result = copy_to_clipboard(text);
        assert!(result.is_ok(), "Failed to copy unicode text: {:?}", result);
    }

    #[test]
    fn test_copy_to_clipboard_newlines() {
        let _guard = clipboard_lock();
        let text = "Line 1\nLine 2\nLine 3\n";
        let result = copy_to_clipboard(text);
        assert!(result.is_ok(), "Failed to copy text with newlines: {:?}", result);
    }

    // ============================================================================
    // Integration Tests for get_selected_text()
    // ============================================================================

    #[test]
    fn test_get_selected_text_after_copy() {
        let _guard = clipboard_lock();
        // Clear clipboard first to ensure we're testing our own copy operation
        copy_to_clipboard("").ok();
        wait_for_clipboard();
        
        let original_text = "Test clipboard read";
        copy_to_clipboard(original_text).expect("Failed to copy text");
        wait_for_clipboard();
        
        let result = get_selected_text();
        assert!(result.is_some(), "Failed to read clipboard after copy");
        assert_eq!(result.unwrap(), original_text, "Read text doesn't match what we copied");
    }

    #[test]
    fn test_get_selected_text_unicode() {
        let _guard = clipboard_lock();
        let original_text = "Hello 👋 World 🌍";
        
        copy_to_clipboard(original_text).expect("Failed to copy unicode text");
        wait_for_clipboard();
        
        let result = get_selected_text();
        assert!(result.is_some(), "Failed to read unicode text from clipboard");
        assert_eq!(result.unwrap(), original_text);
    }

    #[test]
    fn test_get_selected_text_with_newlines() {
        let _guard = clipboard_lock();
        let original_text = "Line 1\nLine 2\nLine 3";
        
        copy_to_clipboard(original_text).expect("Failed to copy text with newlines");
        wait_for_clipboard();
        
        let result = get_selected_text();
        assert!(result.is_some(), "Failed to read text with newlines from clipboard");
        assert_eq!(result.unwrap(), original_text);
    }

    // ============================================================================
    // Round-trip Integration Tests
    // ============================================================================

    #[test]
    fn test_round_trip_ascii() {
        let _guard = clipboard_lock();
        let original_text = "Simple ASCII text for round-trip test";
        
        copy_to_clipboard(original_text).expect("Failed to copy");
        wait_for_clipboard();
        
        let read_text = get_selected_text().expect("Failed to read");
        assert_eq!(read_text, original_text, "Round-trip failed for ASCII text");
    }

    #[test]
    fn test_round_trip_unicode() {
        let _guard = clipboard_lock();
        let original_text = "Hello 👋 World 🌍 Привет мир 你好";
        
        copy_to_clipboard(original_text).expect("Failed to copy unicode");
        wait_for_clipboard();
        
        let read_text = get_selected_text().expect("Failed to read unicode");
        assert_eq!(read_text, original_text, "Round-trip failed for unicode text");
    }

    #[test]
    fn test_round_trip_special_characters() {
        let _guard = clipboard_lock();
        let original_text = "Text with \"quotes\", 'apostrophes',\nnewlines,\tand\ttabs";
        
        copy_to_clipboard(original_text).expect("Failed to copy special chars");
        wait_for_clipboard();
        
        let read_text = get_selected_text().expect("Failed to read special chars");
        assert_eq!(read_text, original_text, "Round-trip failed for special characters");
    }

    #[test]
    fn test_round_trip_long_text() {
        let _guard = clipboard_lock();
        let original_text = "a".repeat(2000);
        
        copy_to_clipboard(&original_text).expect("Failed to copy long text");
        wait_for_clipboard();
        
        let read_text = get_selected_text().expect("Failed to read long text");
        assert_eq!(read_text, original_text, "Round-trip failed for long text");
    }

    #[test]
    fn test_round_trip_multiline() {
        let _guard = clipboard_lock();
        let original_text = "Line 1\nLine 2\nLine 3\nLine 4";
        
        copy_to_clipboard(original_text).expect("Failed to copy multiline text");
        wait_for_clipboard();
        
        let read_text = get_selected_text().expect("Failed to read multiline text");
        assert_eq!(read_text, original_text, "Round-trip failed for multiline text");
    }

    #[test]
    fn test_round_trip_empty_string() {
        let _guard = clipboard_lock();
        let original_text = "";
        
        copy_to_clipboard(original_text).expect("Failed to copy empty string");
        wait_for_clipboard();
        
        // Empty clipboard might return None or empty string
        let read_text = get_selected_text();
        // On macOS/Linux, empty clipboard might return None
        // This is acceptable behavior
        if let Some(text) = read_text {
            assert_eq!(text, "", "Empty string round-trip failed");
        }
        // If None, that's also acceptable for empty clipboard
    }

    // ============================================================================
    // Error Handling Tests
    // ============================================================================

    #[test]
    fn test_copy_to_clipboard_error_message_format() {
        let _guard = clipboard_lock();
        // This test verifies that error messages are descriptive
        // We can't easily simulate a failure, but we can verify successful
        // operations return proper Ok(()) results
        let text = "Test error message format";
        let result = copy_to_clipboard(text);
        
        match result {
            Ok(()) => {
                // Success case - this is expected
            }
            Err(e) => {
                // If it fails, error should be descriptive
                assert!(!e.is_empty(), "Error message should not be empty");
                assert!(e.contains("clipboard"), 
                    "Error message should mention clipboard operation: {}", e);
            }
        }
    }

    #[test]
    fn test_get_selected_text_empty_clipboard() {
        let _guard = clipboard_lock();
        // Clear clipboard by copying empty string
        copy_to_clipboard("").ok();
        wait_for_clipboard();
        
        // Try to read - should return None for empty clipboard
        let result = get_selected_text();
        // On macOS/Linux, empty clipboard typically returns None
        // This is acceptable behavior
        if let Some(text) = result {
            assert_eq!(text, "", "Empty clipboard should return empty string or None");
        }
    }

    #[test]
    fn test_sequential_operations() {
        let _guard = clipboard_lock();
        // Test that multiple clipboard operations work correctly in sequence
        let texts = vec![
            "First text",
            "Second text",
            "Third text",
        ];
        
        for text in texts {
            copy_to_clipboard(text).expect("Failed to copy in sequence");
            wait_for_clipboard();
            
            let read_text = get_selected_text().expect("Failed to read in sequence");
            assert_eq!(read_text, text, "Sequential operation failed");
        }
    }
}

