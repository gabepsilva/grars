//! Windows-specific text extraction implementation using Windows.Media.Ocr

use std::path::Path;
use tracing::{debug, error, info, warn};

/// Extracts text from an image on Windows using the built-in Windows.Media.Ocr API.
/// This is similar to macOS Vision framework - no external dependencies required.
pub(super) fn extract_text_from_image_windows(image_path: &str) -> Result<String, String> {
    info!(path = %image_path, "Starting text extraction from image on Windows using native OCR");
    
    // Verify the image file exists
    if !Path::new(image_path).exists() {
        error!(path = %image_path, "Image file does not exist");
        return Err(format!("Image file does not exist: {}", image_path));
    }
    
    // Initialize Windows Runtime (required for WinRT APIs)
    // WinRT APIs require COM to be initialized in STA mode
    unsafe {
        let hr = windows::Win32::System::Com::CoInitializeEx(
            None,
            windows::Win32::System::Com::COINIT_APARTMENTTHREADED,
        );
        if hr.is_err() {
            // If already initialized, that's okay (S_FALSE = 0x00000001)
            if hr.0 != 0x00000001 {
                error!(hr = hr.0, "Failed to initialize Windows Runtime");
                return Err(format!("Failed to initialize Windows Runtime: HRESULT 0x{:08X}", hr.0));
            }
        }
    }
    
    // Use Windows.Media.Ocr API
    let result = extract_text_with_windows_ocr(image_path);
    
    // Cleanup COM
    unsafe {
        windows::Win32::System::Com::CoUninitialize();
    }
    
    result
}

fn extract_text_with_windows_ocr(image_path: &str) -> Result<String, String> {
    use windows::{
        core::*,
        Graphics::Imaging::*,
        Media::Ocr::*,
        Storage::*,
        Storage::Streams::*,
    };
    
    // Convert image path to absolute Windows path
    let file_path = Path::new(image_path)
        .canonicalize()
        .map_err(|e| {
            error!(error = %e, "Failed to canonicalize image path");
            format!("Failed to canonicalize image path: {}", e)
        })?;
    
    // Convert to Windows path string (backslashes)
    let file_path_str = file_path.to_string_lossy().replace('/', "\\");
    let file_path_hstring: HSTRING = file_path_str.into();
    
    // Get the file using the absolute path
    let file = StorageFile::GetFileFromPathAsync(&file_path_hstring)
        .map_err(|e| {
            error!(error = %e, "Failed to get file from path");
            format!("Failed to open image file: {}", e)
        })?
        .get()
        .map_err(|e| {
            error!(error = %e, "Failed to get StorageFile result");
            format!("Failed to open image file: {}", e)
        })?;
    
    // Open the file stream
    let file_stream = file
        .OpenAsync(FileAccessMode::Read)
        .map_err(|e| {
            error!(error = %e, "Failed to open file stream");
            format!("Failed to open image file: {}", e)
        })?
        .get()
        .map_err(|e| {
            error!(error = %e, "Failed to get file stream result");
            format!("Failed to open image file: {}", e)
        })?;
    
    // Create random access stream reference
    let random_access_stream: IRandomAccessStream = file_stream.cast().map_err(|e| {
        error!(error = %e, "Failed to cast to IRandomAccessStream");
        format!("Failed to process image: {}", e)
    })?;
    
    // Decode the image
    let decoder = BitmapDecoder::CreateAsync(&random_access_stream)
        .map_err(|e| {
            error!(error = %e, "Failed to create bitmap decoder");
            format!("Failed to decode image: {}", e)
        })?
        .get()
        .map_err(|e| {
            error!(error = %e, "Failed to get decoder result");
            format!("Failed to decode image: {}", e)
        })?;
    
    // Get the software bitmap
    let software_bitmap = decoder
        .GetSoftwareBitmapAsync()
        .map_err(|e| {
            error!(error = %e, "Failed to get software bitmap");
            format!("Failed to process image: {}", e)
        })?
        .get()
        .map_err(|e| {
            error!(error = %e, "Failed to get software bitmap result");
            format!("Failed to process image: {}", e)
        })?;
    
    // Create OCR engine with user's profile languages (automatically detects available languages)
    let ocr_engine = OcrEngine::TryCreateFromUserProfileLanguages()
        .map_err(|e| {
            error!(error = %e, "Failed to create OCR engine");
            format!("Failed to initialize OCR engine: {}", e)
        })?;
    
    debug!("OCR engine created successfully");
    
    // Recognize text from the bitmap
    let ocr_result = ocr_engine
        .RecognizeAsync(&software_bitmap)
        .map_err(|e| {
            error!(error = %e, "Failed to recognize text");
            format!("Failed to recognize text: {}", e)
        })?
        .get()
        .map_err(|e| {
            error!(error = %e, "Failed to get OCR result");
            format!("Failed to recognize text: {}", e)
        })?;
    
    // Extract all text lines
    let lines = ocr_result.Lines().map_err(|e| {
        error!(error = %e, "Failed to get OCR lines");
        format!("Failed to extract text: {}", e)
    })?;
    
    let mut extracted_text_parts = Vec::new();
    let line_count = lines.Size().map_err(|e| {
        error!(error = %e, "Failed to get lines count");
        format!("Failed to extract text: {}", e)
    })?;
    
    for i in 0..line_count {
        let line = lines.GetAt(i).map_err(|e| {
            error!(error = %e, line_index = i, "Failed to get OCR line");
            format!("Failed to extract text: {}", e)
        })?;
        
        let text = line.Text().map_err(|e| {
            error!(error = %e, line_index = i, "Failed to get line text");
            format!("Failed to extract text: {}", e)
        })?;
        
        let text_str = text.to_string();
        if !text_str.trim().is_empty() {
            extracted_text_parts.push(text_str);
        }
    }
    
    // Join all text parts with spaces
    let extracted_text = extracted_text_parts.join(" ");
    
    if extracted_text.trim().is_empty() {
        warn!("No text found in image");
        return Err("No text found in image".to_string());
    }
    
    info!(
        bytes = extracted_text.len(),
        lines = line_count,
        "Text extracted successfully from image using Windows OCR"
    );
    debug!(
        text = %extracted_text.chars().take(100).collect::<String>(),
        "Extracted text preview"
    );
    
    Ok(extracted_text)
}
