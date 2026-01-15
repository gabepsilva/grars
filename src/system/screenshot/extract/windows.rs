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
        // If already initialized (S_FALSE = 0x00000001), that's okay
        if hr.is_err() && hr.0 != 0x00000001 {
            error!(hr = hr.0, "Failed to initialize Windows Runtime");
            return Err(format!("Failed to initialize Windows Runtime: HRESULT 0x{:08X}", hr.0));
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
    use std::fs;
    use windows::{
        core::*,
        Graphics::Imaging::*,
        Media::Ocr::*,
        Storage::Streams::*,
    };
    
    // Read the image file into memory
    let image_bytes = fs::read(image_path).map_err(|e| {
        error!(error = %e, "Failed to read image file");
        format!("Failed to read image file: {}", e)
    })?;
    
    debug!(bytes = image_bytes.len(), "Read image file into memory");
    
    // Create an in-memory random access stream from the bytes
    let stream = InMemoryRandomAccessStream::new().map_err(|e| {
        error!(error = %e, "Failed to create in-memory stream");
        format!("Failed to create stream: {}", e)
    })?;
    
    // Create a DataWriter associated with the stream
    let data_writer = DataWriter::CreateDataWriter(&stream).map_err(|e| {
        error!(error = %e, "Failed to create data writer");
        format!("Failed to create stream: {}", e)
    })?;
    
    // Write the image bytes to the stream
    data_writer.WriteBytes(&image_bytes).map_err(|e| {
        error!(error = %e, "Failed to write bytes to stream");
        format!("Failed to write image data: {}", e)
    })?;
    
    // Store the bytes (this commits the write)
    data_writer.StoreAsync().map_err(|e| {
        error!(error = %e, "Failed to store bytes");
        format!("Failed to write image data: {}", e)
    })?
    .get()
    .map_err(|e| {
        error!(error = %e, "Failed to get store result");
        format!("Failed to write image data: {}", e)
    })?;
    
    debug!("Wrote image bytes to stream");
    
    // Reset stream position to beginning for reading
    stream.Seek(0).map_err(|e| {
        error!(error = %e, "Failed to seek stream");
        format!("Failed to process image: {}", e)
    })?;
    
    // Create random access stream reference
    let random_access_stream: IRandomAccessStream = stream.cast().map_err(|e| {
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
    
    // Join all text parts with newlines to preserve line breaks
    let extracted_text = extracted_text_parts.join("\n");
    
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
