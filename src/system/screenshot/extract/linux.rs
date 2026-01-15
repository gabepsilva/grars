//! Linux-specific text extraction implementation

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, error, info, warn};

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

/// Extracts text from an image on Linux using Python script with EasyOCR.
pub(super) fn extract_text_from_image_linux(image_path: &str) -> Result<String, String> {
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
            Path::new("install/extract_text_from_image.py")
                .exists()
                .then(|| Path::new("install/extract_text_from_image.py").to_path_buf())
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
    
    // Preserve all newlines from OCR output - only trim trailing newline from script output
    let extracted_text = String::from_utf8_lossy(&output.stdout).trim_end().to_string();
    
    if extracted_text.is_empty() {
        warn!("No text found in image");
        return Err("No text found in image".to_string());
    }
    
    info!(bytes = extracted_text.len(), "Text extracted successfully from image");
    debug!(text = %extracted_text.chars().take(100).collect::<String>(), "Extracted text preview");
    
    Ok(extracted_text)
}
