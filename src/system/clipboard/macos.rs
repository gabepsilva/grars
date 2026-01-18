//! macOS-specific clipboard implementation using Cmd+C simulation
//!
//! This module implements text selection capture on macOS by simulating Cmd+C.
//! macOS doesn't provide a direct API to read selected text from other applications,
//! so we use AppleScript to send the keystroke to the frontmost application.

use super::process_text;
use arboard::Clipboard;
use macos_accessibility_client::accessibility::application_is_trusted_with_prompt;
use std::process::Command;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Delay before simulating Cmd+C to allow system to settle after hotkey press.
const SETTLE_DELAY_MS: u64 = 100;

/// Delay in AppleScript to allow focus to settle before sending keystroke.
const APPLESCRIPT_FOCUS_DELAY: f64 = 0.05;

/// Maximum time to wait for clipboard to update after Cmd+C simulation.
const CLIPBOARD_POLL_TIMEOUT_MS: u64 = 300;

/// Interval between clipboard polling attempts.
const CLIPBOARD_POLL_INTERVAL_MS: u64 = 50;

/// Check if we have accessibility permissions (macOS only).
///
/// Will prompt the user to grant permissions if not already granted.
/// Returns `true` if permissions are granted, `false` otherwise.
fn check_accessibility_permissions() -> bool {
    let trusted = application_is_trusted_with_prompt();
    if !trusted {
        warn!(
            "Accessibility permissions not granted - enable in System Settings > Privacy & Security > Accessibility"
        );
    }
    trusted
}

/// Simulates Cmd+C using AppleScript to copy selected text from the frontmost application.
///
/// This function uses `osascript` to execute an AppleScript that:
/// 1. Identifies the frontmost application
/// 2. Adds a small delay to allow focus to settle
/// 3. Sends the keystroke "c" with command modifier to that application
///
/// # Errors
///
/// Returns an error if:
/// - `osascript` cannot be executed
/// - The AppleScript execution fails
///
/// # Requirements
///
/// Requires Accessibility permissions to be granted in System Settings.
fn simulate_cmd_c() -> Result<(), String> {
    debug!("Simulating Cmd+C via AppleScript");

    // AppleScript to send Cmd+C to the frontmost application
    let script = format!(
        r#"
        tell application "System Events"
            set frontApp to name of first application process whose frontmost is true
            delay {}
            keystroke "c" using command down
        end tell
    "#,
        APPLESCRIPT_FOCUS_DELAY
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("Failed to execute osascript: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let trimmed = stdout.trim();
        if !trimmed.is_empty() {
            debug!(output = %trimmed, "AppleScript executed successfully");
        } else {
            debug!("AppleScript executed successfully");
        }
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let error_msg = if stderr.trim().is_empty() {
            format!(
                "AppleScript failed with exit code {}",
                output.status.code().unwrap_or(-1)
            )
        } else {
            format!("AppleScript failed: {}", stderr.trim())
        };
        warn!(error = %error_msg, "Failed to simulate Cmd+C");
        Err(error_msg)
    }
}

/// Polls clipboard for new content, checking at regular intervals up to max_wait.
///
/// Returns the text if clipboard has content, `None` if timeout is reached.
///
/// # Arguments
///
/// * `max_wait` - Maximum duration to wait for clipboard to update
fn poll_clipboard_for_text(max_wait: Duration) -> Option<String> {
    let poll_interval = Duration::from_millis(CLIPBOARD_POLL_INTERVAL_MS);
    let mut elapsed = Duration::ZERO;

    while elapsed < max_wait {
        std::thread::sleep(poll_interval);
        elapsed += poll_interval;

        if let Some(text) = Clipboard::new()
            .and_then(|mut cb| cb.get_text())
            .ok()
            .filter(|t| !t.is_empty())
        {
            debug!(
                elapsed_ms = elapsed.as_millis(),
                "Clipboard updated with new content"
            );
            return Some(text);
        }
    }

    debug!(
        timeout_ms = max_wait.as_millis(),
        "Clipboard polling timeout reached"
    );
    None
}

/// Gets the currently selected text on macOS using Cmd+C simulation.
///
/// This function implements a workaround for macOS's lack of direct API access
/// to read selected text from other applications. The process:
///
/// 1. Saves the current clipboard contents
/// 2. Clears the clipboard to detect when copy completes
/// 3. Simulates Cmd+C using AppleScript to copy selected text
/// 4. Polls clipboard until it updates (or timeout)
/// 5. Restores the original clipboard contents
///
/// # Returns
///
/// Returns `Some(text)` if text was successfully captured, `None` if:
/// - Accessibility permissions are not granted
/// - Clipboard access fails
/// - Cmd+C simulation fails
/// - No text is selected or clipboard doesn't update within timeout
///
/// # Requirements
///
/// Requires Accessibility permissions to be granted in System Settings.
pub(super) fn get_selected_text_macos() -> Option<String> {
    debug!("Capturing selected text via Cmd+C simulation");

    // Let system settle after hotkey press
    std::thread::sleep(Duration::from_millis(SETTLE_DELAY_MS));

    // Check accessibility permissions
    if !check_accessibility_permissions() {
        warn!("Cannot capture selected text: Accessibility permissions required");
        return None;
    }

    // Save current clipboard contents
    let mut clipboard = match Clipboard::new() {
        Ok(cb) => cb,
        Err(e) => {
            warn!(error = %e, "Failed to initialize clipboard");
            return None;
        }
    };

    let original_text = clipboard.get_text().ok();

    // Clear clipboard to detect when copy completes
    if let Err(e) = clipboard.clear() {
        warn!(error = %e, "Failed to clear clipboard");
        // Continue anyway - worst case we might not detect the copy
    }

    // Simulate Cmd+C
    if let Err(e) = simulate_cmd_c() {
        warn!(error = %e, "Failed to simulate Cmd+C");
        restore_clipboard(original_text);
        return None;
    }

    // Poll clipboard for new content
    let selected_text = poll_clipboard_for_text(Duration::from_millis(CLIPBOARD_POLL_TIMEOUT_MS));

    if let Some(text) = &selected_text {
        info!(chars = text.len(), "Successfully captured selected text");
    } else {
        debug!("No text selected or clipboard didn't update within timeout");
    }

    // Restore original clipboard contents
    restore_clipboard(original_text);

    // Process and return
    selected_text.and_then(|text| process_text(text, "selected text"))
}

/// Restores clipboard to its original contents.
///
/// # Arguments
///
/// * `original_text` - The original clipboard text to restore, or `None` to clear
fn restore_clipboard(original_text: Option<String>) {
    let Ok(mut clipboard) = Clipboard::new() else {
        warn!("Failed to create clipboard instance for restoration");
        return;
    };

    match original_text {
        Some(text) => {
            let text_len = text.len();
            if let Err(e) = clipboard.set_text(text) {
                warn!(error = %e, "Failed to restore original clipboard contents");
            } else {
                debug!(chars = text_len, "Restored original clipboard contents");
            }
        }
        None => {
            if let Err(e) = clipboard.clear() {
                warn!(error = %e, "Failed to clear clipboard during restoration");
            } else {
                debug!("Cleared clipboard (original was empty)");
            }
        }
    }
}
