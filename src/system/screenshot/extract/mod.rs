//! Text extraction from images functionality

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

/// Extracts text from an image using platform-native OCR APIs.
/// 
/// On macOS, uses Swift script with Vision framework for OCR.
/// On Linux, uses EasyOCR via Python script.
/// On Windows, uses built-in Windows.Media.Ocr API (no external dependencies required).
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
    
    #[cfg(target_os = "windows")]
    {
        windows::extract_text_from_image_windows(image_path)
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        tracing::warn!("Text extraction from images not supported on this platform");
        Err("Text extraction from images is only supported on macOS, Linux, and Windows".to_string())
    }
}
