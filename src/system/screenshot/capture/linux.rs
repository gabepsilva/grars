//! Linux-specific screenshot capture implementation

use std::env;
use std::path::Path;
use std::process::Command;
use tracing::{debug, error, info};

/// Screenshot tool configuration
struct Tool {
    name: &'static str,
    /// Args before the output path (path is appended last)
    args: &'static [&'static str],
}

/// Try to capture with a single-command tool. Returns:
/// - Some(Ok(path)) on success
/// - Some(Err(msg)) if user cancelled
/// - None if tool unavailable or failed (try next)
fn try_tool(tool: &Tool, output_path: &Path) -> Option<Result<String, String>> {
    // Check if tool is available
    if Command::new(tool.name).arg("--version").output().is_err() {
        return None;
    }
    
    info!("Using {} for screenshot capture", tool.name);
    
    let output = Command::new(tool.name)
        .args(tool.args)
        .arg(output_path.as_os_str())
        .output();
    
    match output {
        Ok(output) => {
            if !output.status.success() {
                let exit_code = output.status.code().unwrap_or(-1);
                // Exit code 1 typically means user cancelled
                if exit_code == 1 {
                    debug!("User cancelled screenshot selection");
                    return Some(Err("Screenshot selection cancelled".to_string()));
                }
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!(code = exit_code, stderr = %stderr.trim(), "{} command failed", tool.name);
                None // Try next tool
            } else if output_path.exists() {
                let path_str = output_path.to_string_lossy().to_string();
                info!(path = %path_str, "Screenshot captured successfully with {}", tool.name);
                Some(Ok(path_str))
            } else {
                None
            }
        }
        Err(e) => {
            debug!(error = %e, "{} execution failed, trying next tool", tool.name);
            None
        }
    }
}

/// Special handler for grim+slurp (Wayland) which requires two commands
fn try_grim_slurp(output_path: &Path) -> Option<Result<String, String>> {
    // Both tools must be available
    if Command::new("grim").arg("--version").output().is_err() 
        || Command::new("slurp").arg("--version").output().is_err() {
        return None;
    }
    
    info!("Using grim+slurp for screenshot capture");
    
    // First, get the region using slurp
    let slurp_output = match Command::new("slurp").output() {
        Ok(o) => o,
        Err(e) => {
            debug!(error = %e, "slurp execution failed, trying next tool");
            return None;
        }
    };
    
    if !slurp_output.status.success() {
        if slurp_output.status.code() == Some(1) {
            debug!("User cancelled screenshot selection");
            return Some(Err("Screenshot selection cancelled".to_string()));
        }
        return None;
    }
    
    let region = String::from_utf8_lossy(&slurp_output.stdout).trim().to_string();
    if region.is_empty() {
        return None;
    }
    
    // Capture the selected region with grim
    match Command::new("grim").arg("-g").arg(&region).arg(output_path.as_os_str()).output() {
        Ok(grim_output) if grim_output.status.success() && output_path.exists() => {
            let path_str = output_path.to_string_lossy().to_string();
            info!(path = %path_str, "Screenshot captured successfully with grim+slurp");
            Some(Ok(path_str))
        }
        Ok(_) => None,
        Err(e) => {
            debug!(error = %e, "grim execution failed");
            None
        }
    }
}

/// Captures a screenshot region on Linux using available screenshot tools.
pub(super) fn capture_region_linux() -> Result<String, String> {
    info!("Starting interactive screenshot region selection on Linux");
    
    let screenshot_path = env::temp_dir().join("insight-reader-screenshot.png");
    debug!(path = %screenshot_path.display(), "Screenshot will be saved to temp file");
    
    // Tools in order of preference
    const TOOLS: &[Tool] = &[
        Tool { name: "flameshot", args: &["gui", "--path"] },
        Tool { name: "maim", args: &["-s"] },
        // grim+slurp handled separately
        Tool { name: "scrot", args: &["-s"] },
        Tool { name: "gnome-screenshot", args: &["-a", "--file"] },
        Tool { name: "spectacle", args: &["-r", "-b", "-n", "-o"] },
    ];
    
    // Try flameshot and maim first
    for tool in &TOOLS[..2] {
        if let Some(result) = try_tool(tool, &screenshot_path) {
            return result;
        }
    }
    
    // Try grim+slurp (Wayland)
    if let Some(result) = try_grim_slurp(&screenshot_path) {
        return result;
    }
    
    // Try remaining tools
    for tool in &TOOLS[2..] {
        if let Some(result) = try_tool(tool, &screenshot_path) {
            return result;
        }
    }
    
    error!("No screenshot tools found. Please install one of: flameshot, maim, grim+slurp, scrot, gnome-screenshot, or spectacle");
    Err("No screenshot tools available. Please install flameshot, maim, grim+slurp, scrot, gnome-screenshot, or spectacle".to_string())
}
