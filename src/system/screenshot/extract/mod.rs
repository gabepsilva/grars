//! Text extraction from images functionality

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

/// Extracts text from an image using macOS Vision framework or Linux EasyOCR.
/// 
/// On macOS, uses AppleScript to call the Vision framework for OCR.
/// On Linux, uses EasyOCR via Python script.
/// Returns the extracted text, or an error message.
pub fn extract_text_from_image(image_path: &str) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        macos::extract_text_from_image_macos(image_path)
    }
    
    #[cfg(target_os = "linux")]
    {
        linux::extract_text_from_image_linux(image_path)
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        tracing::warn!("Text extraction from images not supported on this platform");
        Err("Text extraction from images is only supported on macOS and Linux".to_string())
    }
}
