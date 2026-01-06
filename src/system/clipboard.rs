/// Clipboard and selection reading utilities

use std::process::Command;

/// Gets the currently selected text (PRIMARY selection) on Linux.
/// Uses wl-paste for Wayland, xclip for X11.
pub fn get_selected_text() -> Option<String> {
    let try_cmd = |cmd: &str, args: &[&str]| -> Option<String> {
        let output = Command::new(cmd).args(args).output().ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&output.stdout);
        (!text.is_empty()).then(|| text.into_owned())
    };

    // Try wl-paste first (Wayland), fallback to xclip (X11)
    try_cmd("wl-paste", &["--primary", "--no-newline"])
        .or_else(|| try_cmd("xclip", &["-selection", "primary", "-o"]))
}


