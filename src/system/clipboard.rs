//! Clipboard and selection reading utilities

use std::process::Command;

use tracing::{debug, trace};

/// Gets the currently selected text (PRIMARY selection) on Linux.
/// Uses wl-paste for Wayland, xclip for X11.
pub fn get_selected_text() -> Option<String> {
    let try_cmd = |cmd: &str, args: &[&str]| -> Option<String> {
        trace!(cmd, ?args, "Trying clipboard command");
        let output = Command::new(cmd).args(args).output().ok()?;
        if !output.status.success() {
            debug!(cmd, code = ?output.status.code(), "Clipboard command failed");
            return None;
        }
        let text = String::from_utf8_lossy(&output.stdout);
        (!text.is_empty()).then(|| text.into_owned())
    };

    // Try wl-paste first (Wayland), fallback to xclip (X11)
    let result = try_cmd("wl-paste", &["--primary", "--no-newline"])
        .or_else(|| try_cmd("xclip", &["-selection", "primary", "-o"]));

    if result.is_none() {
        debug!("No text available from clipboard/selection");
    }

    result
}


