//! Clipboard and selection reading utilities

use std::process::Command;

use tracing::{debug, info, trace, warn};

/// Creates a preview string for logging (first 200 chars).
fn text_preview(text: &str) -> String {
    if text.chars().count() > 200 {
        text.chars().take(200).collect::<String>() + "..."
    } else {
        text.to_string()
    }
}

/// Restores text to the clipboard.
fn restore_clipboard(text: &str) -> Result<(), std::io::Error> {
    use std::io::Write;
    let mut child = Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())?;
        stdin.flush()?;
    }
    
    child.wait()?;
    Ok(())
}

/// Gets the currently selected text.
/// - On Linux: Uses wl-paste for Wayland, xclip for X11 (PRIMARY selection)
/// - On macOS: Uses AppleScript to copy selection to clipboard, then reads it
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
    info!("Attempting to read selected text on macOS");
    
    // Strategy: Save current clipboard, copy selection, read clipboard, restore clipboard
    // This requires accessibility permissions but is the most reliable method
    
    // Step 1: Save current clipboard
    let saved_clipboard = Command::new("pbpaste")
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                (!text.is_empty()).then_some(text)
            } else {
                None
            }
        });
    
    // Step 2: Copy selection to clipboard using AppleScript
    // This simulates Cmd+C which requires accessibility permissions
    let copy_script = r#"
        tell application "System Events"
            keystroke "c" using command down
        end tell
    "#;
    
    let copy_output = Command::new("osascript")
        .arg("-e")
        .arg(copy_script)
        .output();
    
    match copy_output {
        Ok(output) if output.status.success() => {
            // Step 3: Small delay to ensure clipboard is updated
            std::thread::sleep(std::time::Duration::from_millis(50));
            
            // Step 4: Read clipboard
            let clipboard_text = Command::new("pbpaste")
                .output()
                .ok()
                .and_then(|output| {
                    if output.status.success() {
                        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        (!text.is_empty()).then_some(text)
                    } else {
                        None
                    }
                });
            
            // Step 5: Restore original clipboard if we saved one
            if let Some(ref saved) = saved_clipboard {
                if let Err(e) = restore_clipboard(saved) {
                    debug!(error = %e, "Failed to restore clipboard");
                }
            }
            
            if let Some(ref text) = clipboard_text {
                info!(bytes = text.len(), "Successfully retrieved selected text");
                debug!(text = %text_preview(text), "Captured text content");
            } else {
                debug!("No text in clipboard after copy attempt (no text selected or accessibility permissions not granted)");
            }
            
            clipboard_text
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!(
                code = ?output.status.code(),
                stderr = %stderr.trim(),
                "AppleScript copy command failed (may need accessibility permissions)"
            );
            None
        }
        Err(e) => {
            warn!(error = %e, "Failed to execute osascript command");
            None
        }
    }
}


