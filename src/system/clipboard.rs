//! Clipboard and selection reading utilities

use std::process::Command;

use tracing::{debug, info, trace, warn};

/// Gets the currently selected text (PRIMARY selection) on Linux.
/// Uses wl-paste for Wayland, xclip for X11.
pub fn get_selected_text() -> Option<String> {
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
    } else {
        warn!("No text available from clipboard/selection (no text selected or commands failed)");
    }

    result
}


