//! Clipboard and selection reading utilities

use tracing::{debug, info, warn};
#[cfg(target_os = "linux")]
use std::process::Command;
#[cfg(target_os = "linux")]
use tracing::trace;

/// Creates a preview string for logging (first 200 chars).
pub(crate) fn text_preview(text: &str) -> String {
    if text.chars().count() > 200 {
        format!("{}...", text.chars().take(200).collect::<String>())
    } else {
        text.to_string()
    }
}

/// Gets the currently selected text.
/// - On Linux: Uses wl-paste for Wayland, xclip for X11 (PRIMARY selection)
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

#[cfg(target_os = "linux")]
fn get_selected_text_linux() -> Option<String> {
    let try_cmd = |cmd: &str, args: &[&str]| -> Option<String> {
        trace!(cmd, ?args, "Trying clipboard command");
        
        let output = match Command::new(cmd).args(args).output() {
            Ok(output) => output,
            Err(e) => {
                warn!(cmd, error = %e, "Failed to execute clipboard command");
                return None;
            }
        };
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!(
                cmd,
                code = ?output.status.code(),
                stderr = %stderr.trim(),
                "Clipboard command failed"
            );
            return None;
        }
        
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.is_empty() {
            trace!(cmd, "Clipboard command returned empty text");
            return None;
        }
        
        Some(text)
    };

    // Try wl-paste first (Wayland), fallback to xclip (X11)
    info!("Attempting to read selected text from clipboard/selection");
    let result = try_cmd("wl-paste", &["--primary", "--no-newline"])
        .or_else(|| {
            debug!("wl-paste failed, trying xclip");
            try_cmd("xclip", &["-selection", "primary", "-o"])
        });

    if let Some(ref text) = result {
        info!(bytes = text.len(), "Successfully retrieved selected text");
        debug!(text = %text_preview(text), "Captured text content");
    } else {
        warn!("No text available from clipboard/selection (no text selected or commands failed)");
    }

    result
}

#[cfg(target_os = "macos")]
fn get_selected_text_macos() -> Option<String> {
    use arboard::Clipboard;
    
    info!("Attempting to read text from macOS clipboard using arboard");
    
    let mut clipboard = match Clipboard::new() {
        Ok(clipboard) => clipboard,
        Err(e) => {
            warn!(error = %e, "Failed to initialize clipboard");
            return None;
        }
    };
    
    match clipboard.get_text() {
        Ok(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                debug!("Clipboard is empty");
                None
            } else {
                info!(bytes = trimmed.len(), "Successfully retrieved text from clipboard");
                debug!(text = %text_preview(trimmed), "Captured text content");
                Some(trimmed.to_owned())
            }
        }
        Err(e) => {
            warn!(error = %e, "Failed to read from clipboard");
            None
        }
    }
}

/// Copies text to the clipboard.
/// - On macOS: Uses arboard
/// - On Linux: Uses wl-copy for Wayland, xclip for X11
/// - On other platforms: Returns an error
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        copy_to_clipboard_macos(text)
    }
    
    #[cfg(target_os = "linux")]
    {
        copy_to_clipboard_linux(text)
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        warn!("Platform not supported for clipboard copy");
        Err("Clipboard copy not supported on this platform".to_string())
    }
}

#[cfg(target_os = "macos")]
fn copy_to_clipboard_macos(text: &str) -> Result<(), String> {
    use arboard::Clipboard;
    
    info!("Copying text to macOS clipboard using arboard");
    
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

#[cfg(target_os = "linux")]
fn copy_to_clipboard_linux(text: &str) -> Result<(), String> {
    use std::io::Write;
    
    let try_cmd = |cmd: &str, args: &[&str]| -> Result<(), String> {
        trace!(cmd, ?args, "Trying clipboard copy command");
        
        let mut child = match Command::new(cmd).args(args).stdin(std::process::Stdio::piped()).spawn() {
            Ok(child) => child,
            Err(e) => {
                warn!(cmd, error = %e, "Failed to execute clipboard copy command");
                return Err(format!("Failed to execute {}: {}", cmd, e));
            }
        };
        
        if let Some(mut stdin) = child.stdin.take() {
            if let Err(e) = stdin.write_all(text.as_bytes()) {
                warn!(cmd, error = %e, "Failed to write to clipboard command stdin");
                return Err(format!("Failed to write to {}: {}", cmd, e));
            }
        }
        
        match child.wait() {
            Ok(status) => {
                if status.success() {
                    info!(cmd, bytes = text.len(), "Successfully copied text to clipboard");
                    Ok(())
                } else {
                    let stderr = format!("{} exited with code: {:?}", cmd, status.code());
                    warn!(cmd, %stderr, "Clipboard copy command failed");
                    Err(stderr)
                }
            }
            Err(e) => {
                warn!(cmd, error = %e, "Failed to wait for clipboard copy process");
                Err(format!("Failed to wait for {}: {}", cmd, e))
            }
        }
    };
    
    // Try wl-copy first (Wayland), fallback to xclip (X11)
    info!("Attempting to copy text to clipboard");
    try_cmd("wl-copy", &[])
        .or_else(|_| {
            debug!("wl-copy failed, trying xclip");
            try_cmd("xclip", &["-selection", "clipboard"])
        })
}

#[cfg(test)]
#[cfg(target_os = "macos")]
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
        // On macOS, empty clipboard might return None
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
        // On macOS, empty clipboard typically returns None
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

