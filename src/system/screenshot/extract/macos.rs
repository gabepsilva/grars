//! macOS-specific text extraction implementation

use std::env;
use std::path::Path;
use std::process::Command;
use tracing::{debug, error, info, warn};

/// Extracts text from an image on macOS using Swift script with Vision framework.
pub(super) fn extract_text_from_image_macos(image_path: &str) -> Result<String, String> {
    info!(path = %image_path, "Starting text extraction from image");
    
    // Verify the image file exists
    if !Path::new(image_path).exists() {
        error!(path = %image_path, "Image file does not exist");
        return Err(format!("Image file does not exist: {}", image_path));
    }
    
    // Find the Swift script path: try multiple locations
    let script_path = env::current_exe()
        .ok()
        .and_then(|exe_path| {
            // Try app bundle Resources directory (if running from app bundle)
            exe_path
                .parent()
                .and_then(|macos_dir| {
                    macos_dir.parent().map(|contents| {
                        contents.join("Resources").join("extract_text_from_image.swift")
                    })
                })
                .filter(|p| p.exists())
        })
        .or_else(|| {
            // Try standard installation directory
            env::var("HOME")
                .ok()
                .map(|home| {
                    Path::new(&home)
                        .join(".local")
                        .join("share")
                        .join("insight-reader")
                        .join("bin")
                        .join("extract_text_from_image.swift")
                })
                .filter(|p| p.exists())
        })
        .or_else(|| {
            // Try executable directory
            env::current_exe()
                .ok()
                .and_then(|exe_path| {
                    exe_path
                        .parent()
                        .map(|dir| dir.join("extract_text_from_image.swift"))
                        .filter(|p| p.exists())
                })
        })
        .or_else(|| {
            // Try parent of executable directory
            env::current_exe()
                .ok()
                .and_then(|exe_path| {
                    exe_path
                        .parent()
                        .and_then(|dir| dir.parent())
                        .map(|dir| dir.join("extract_text_from_image.swift"))
                        .filter(|p| p.exists())
                })
        })
        .or_else(|| {
            // Try relative path from current directory (for development)
            Path::new("install/extract_text_from_image.swift")
                .exists()
                .then(|| Path::new("install/extract_text_from_image.swift").to_path_buf())
        })
        .ok_or_else(|| {
            error!("extract_text_from_image.swift script not found in any expected location");
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
