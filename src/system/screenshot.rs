//! Screenshot and region capture utilities

#[allow(unused_imports)] // These are used in macOS-specific code blocks
use tracing::{debug, error, info, warn};

/// Captures a screenshot of a selected screen region.
/// 
/// On macOS, uses `screencapture -i` for interactive region selection.
/// On Linux, tries multiple screenshot tools in order of preference.
/// Returns the path to the captured image file, or an error message.
pub fn capture_region() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        capture_region_macos()
    }
    
    #[cfg(target_os = "linux")]
    {
        capture_region_linux()
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        warn!("Screenshot region selection not supported on this platform");
        Err("Screenshot region selection is only supported on macOS and Linux".to_string())
    }
}

#[cfg(target_os = "macos")]
fn capture_region_macos() -> Result<String, String> {
    use std::env;
    use std::process::Command;
    
    info!("Starting interactive screenshot region selection");
    
    // Create temporary file path for the screenshot
    let temp_dir = env::temp_dir();
    let screenshot_path = temp_dir.join("insight-reader-screenshot.png");
    
    debug!(path = %screenshot_path.display(), "Screenshot will be saved to temp file");
    
    // Execute screencapture with -i flag for interactive region selection
    // -i: interactive mode (shows crosshair for region selection)
    // The user can press Escape to cancel
    let output = match Command::new("screencapture")
        .arg("-i")
        .arg(screenshot_path.as_os_str())
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            error!(error = %e, "Failed to execute screencapture command");
            return Err(format!("Failed to execute screenshot command: {}", e));
        }
    };
    
    // Check if the command succeeded
    if !output.status.success() {
        let exit_code = output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // Exit code 1 typically means user cancelled (Escape key)
        if exit_code == 1 {
            debug!("User cancelled screenshot selection");
            return Err("Screenshot selection cancelled".to_string());
        }
        
        error!(
            code = exit_code,
            stderr = %stderr.trim(),
            "screencapture command failed"
        );
        return Err(format!("Screenshot failed: {}", stderr.trim()));
    }
    
    // Verify the file was actually created
    if !screenshot_path.exists() {
        error!(path = %screenshot_path.display(), "Screenshot file was not created");
        return Err("Screenshot file was not created".to_string());
    }
    
    // Get the file path as a string
    let path_str = screenshot_path.to_string_lossy().to_string();
    info!(path = %path_str, "Screenshot captured successfully");
    
    Ok(path_str)
}

#[cfg(target_os = "linux")]
fn capture_region_linux() -> Result<String, String> {
    use std::env;
    use std::path::Path;
    use std::process::Command;
    
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
        Tool { name: "spectacle", args: &["-r", "-b", "-o"] },
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

/// Extracts text from an image using macOS Vision framework or Linux EasyOCR.
/// 
/// On macOS, uses AppleScript to call the Vision framework for OCR.
/// On Linux, uses EasyOCR via Python script.
/// Returns the extracted text, or an error message.
pub fn extract_text_from_image(image_path: &str) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        extract_text_from_image_macos(image_path)
    }
    
    #[cfg(target_os = "linux")]
    {
        extract_text_from_image_linux(image_path)
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        warn!("Text extraction from images not supported on this platform");
        Err("Text extraction from images is only supported on macOS and Linux".to_string())
    }
}

#[cfg(target_os = "macos")]
fn extract_text_from_image_macos(image_path: &str) -> Result<String, String> {
    use std::env;
    use std::path::Path;
    use std::process::Command;
    
    info!(path = %image_path, "Starting text extraction from image");
    
    // Verify the image file exists
    if !Path::new(image_path).exists() {
        error!(path = %image_path, "Image file does not exist");
        return Err(format!("Image file does not exist: {}", image_path));
    }
    
    // Find the Swift script path: try executable directory, parent, then current directory
    let script_path = env::current_exe()
        .ok()
        .and_then(|exe_path| {
            exe_path.parent()
                .map(|dir| dir.join("extract_text_from_image.swift"))
                .filter(|p| p.exists())
        })
        .or_else(|| {
            env::current_exe()
                .ok()
                .and_then(|exe_path| {
                    exe_path.parent()
                        .and_then(|dir| dir.parent())
                        .map(|dir| dir.join("extract_text_from_image.swift"))
                        .filter(|p| p.exists())
                })
        })
        .or_else(|| {
            Path::new("extract_text_from_image.swift")
                .exists()
                .then(|| Path::new("extract_text_from_image.swift").to_path_buf())
        })
        .ok_or_else(|| {
            error!("extract_text_from_image.swift script not found");
            "extract_text_from_image.swift script not found".to_string()
        })?;
    
    debug!(script = %script_path.display(), "Using Swift script for text extraction");
    
    // Execute Swift script
    let output = match Command::new("swift")
        .arg(script_path.as_os_str())
        .arg(image_path)
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            error!(error = %e, "Failed to execute swift command");
            return Err(format!("Failed to execute text extraction: {}", e));
        }
    };
    
    // Check if the command succeeded
    if !output.status.success() {
        let exit_code = output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // Exit code 1 might mean "no text found" (which is not an error)
        // Check if stderr contains an actual error message
        if exit_code == 1 && stderr.trim().is_empty() {
            warn!("No text found in image");
            return Err("No text found in image".to_string());
        }
        
        error!(
            code = exit_code,
            stderr = %stderr.trim(),
            "Text extraction failed"
        );
        return Err(format!("Text extraction failed: {}", stderr.trim()));
    }
    
    let extracted_text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    if extracted_text.is_empty() {
        warn!("No text found in image");
        return Err("No text found in image".to_string());
    }
    
    info!(bytes = extracted_text.len(), "Text extracted successfully from image");
    debug!(text = %extracted_text.chars().take(100).collect::<String>(), "Extracted text preview");
    
    Ok(extracted_text)
}

#[cfg(target_os = "linux")]
fn extract_text_from_image_linux(image_path: &str) -> Result<String, String> {
    use std::env;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    
    use dirs;
    
    /// Find Python interpreter in the venv (same location as piper binary)
    fn find_venv_python() -> Option<PathBuf> {
        // Check project-local virtualenv first (development)
        if let Ok(current_dir) = env::current_dir() {
            let project_python = current_dir.join("venv").join("bin").join("python");
            if project_python.exists() {
                return Some(project_python);
            }
        }
        
        // Check user installation (XDG Base Directory standard: ~/.local/share/insight-reader)
        if let Some(data_dir) = dirs::data_dir() {
            let user_python = data_dir.join("insight-reader").join("venv").join("bin").join("python");
            if user_python.exists() {
                return Some(user_python);
            }
        }
        
        None
    }
    
    info!(path = %image_path, "Starting text extraction from image on Linux");
    
    // Verify the image file exists
    if !Path::new(image_path).exists() {
        error!(path = %image_path, "Image file does not exist");
        return Err(format!("Image file does not exist: {}", image_path));
    }
    
    // Find the Python script path: try executable directory, parent, then current directory
    let script_path = env::current_exe()
        .ok()
        .and_then(|exe_path| {
            exe_path.parent()
                .map(|dir| dir.join("extract_text_from_image.py"))
                .filter(|p| p.exists())
        })
        .or_else(|| {
            env::current_exe()
                .ok()
                .and_then(|exe_path| {
                    exe_path.parent()
                        .and_then(|dir| dir.parent())
                        .map(|dir| dir.join("extract_text_from_image.py"))
                        .filter(|p| p.exists())
                })
        })
        .or_else(|| {
            // Check in XDG data directory bin folder (standard installation location: ~/.local/share/insight-reader/bin/)
            dirs::data_dir()
                .map(|data_dir| data_dir.join("insight-reader").join("bin").join("extract_text_from_image.py"))
                .filter(|p| p.exists())
        })
        .or_else(|| {
            // Check current directory (development)
            Path::new("extract_text_from_image.py")
                .exists()
                .then(|| Path::new("extract_text_from_image.py").to_path_buf())
        })
        .ok_or_else(|| {
            error!("extract_text_from_image.py script not found");
            "extract_text_from_image.py script not found".to_string()
        })?;
    
    debug!(script = %script_path.display(), "Using Python script for text extraction");
    
    // Find Python interpreter in venv (same location as piper binary)
    let python_interpreter = find_venv_python()
        .unwrap_or_else(|| {
            warn!("Venv Python not found, falling back to system python3");
            PathBuf::from("python3")
        });
    
    debug!(python = %python_interpreter.display(), "Using Python interpreter for text extraction");
    
    // Execute Python script
    let output = match Command::new(&python_interpreter)
        .arg(script_path.as_os_str())
        .arg(image_path)
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            error!(error = %e, "Failed to execute python3 command");
            return Err(format!("Failed to execute text extraction: {}", e));
        }
    };
    
    // Check if the command succeeded
    if !output.status.success() {
        let exit_code = output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // Exit code 1 might mean "no text found" (which is not an error)
        // Check if stderr contains an actual error message
        if exit_code == 1 && stderr.trim().is_empty() {
            warn!("No text found in image");
            return Err("No text found in image".to_string());
        }
        
        error!(
            code = exit_code,
            stderr = %stderr.trim(),
            "Text extraction failed"
        );
        return Err(format!("Text extraction failed: {}", stderr.trim()));
    }
    
    let extracted_text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    if extracted_text.is_empty() {
        warn!("No text found in image");
        return Err("No text found in image".to_string());
    }
    
    info!(bytes = extracted_text.len(), "Text extracted successfully from image");
    debug!(text = %extracted_text.chars().take(100).collect::<String>(), "Extracted text preview");
    
    Ok(extracted_text)
}
