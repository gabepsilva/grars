//! Screenshot and region capture utilities

#[allow(unused_imports)] // These are used in macOS-specific code blocks
use tracing::{debug, error, info, warn};

/// Captures a screenshot of a selected screen region.
/// 
/// On macOS, uses `screencapture -i` for interactive region selection.
/// Returns the path to the captured image file, or an error message.
pub fn capture_region() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        capture_region_macos()
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        warn!("Screenshot region selection not supported on this platform");
        Err("Screenshot region selection is only supported on macOS".to_string())
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

/// Extracts text from an image using macOS Vision framework.
/// 
/// On macOS, uses AppleScript to call the Vision framework for OCR.
/// Returns the extracted text, or an error message.
#[allow(unused_variables)] // image_path is used in macOS-specific code
pub fn extract_text_from_image(image_path: &str) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        extract_text_from_image_macos(image_path)
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        warn!("Text extraction from images not supported on this platform");
        Err("Text extraction from images is only supported on macOS".to_string())
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
