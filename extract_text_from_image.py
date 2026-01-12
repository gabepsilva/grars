#!/usr/bin/env python3
"""
Extract text from an image using EasyOCR.
Similar to extract_text_from_image.swift for macOS, but uses EasyOCR instead of Vision framework.
"""

import sys
import os

def write_error(message: str) -> None:
    """Write error message to stderr."""
    sys.stderr.write(f"{message}\n")
    sys.stderr.flush()

def main() -> int:
    """Main function to extract text from image."""
    # Check command-line arguments
    if len(sys.argv) != 2:
        write_error("Usage: extract_text_from_image.py <image_path>")
        return 1
    
    image_path = sys.argv[1]
    
    # Verify image file exists
    if not os.path.exists(image_path):
        write_error(f"Error: Image file does not exist: {image_path}")
        return 1
    
    try:
        # Import EasyOCR
        import easyocr
    except ImportError:
        write_error("Error: easyocr module not found. Please install it with: pip install easyocr")
        return 1
    
    try:
        # Initialize EasyOCR reader
        # This will download models on first use (may be slow)
        # EasyOCR supports 80+ languages. Common language codes:
        # en (English), fr (French), de (German), es (Spanish), it (Italian),
        # pt (Portuguese), ch_sim (Simplified Chinese), ch_tra (Traditional Chinese),
        # ja (Japanese), ko (Korean), ru (Russian), uk (Ukrainian), th (Thai), etc.
        # See https://www.jaided.ai/easyocr/ for full list
        # gpu=False uses CPU only (faster install, no CUDA dependencies)
        languages = [
            'en',      # English
            'ch_tra',  # Traditional Chinese
        ]
        reader = easyocr.Reader(languages, gpu=False)
        
        # Read text from image
        results = reader.readtext(image_path)
        
        # Extract text from results
        # results is a list of tuples: (bbox, text, confidence)
        extracted_text_parts = []
        for (bbox, text, confidence) in results:
            extracted_text_parts.append(text)
        
        # Join all text parts with spaces
        extracted_text = " ".join(extracted_text_parts)
        
        if not extracted_text.strip():
            # No text found - exit with code 1 but no error message (this is expected)
            return 1
        
        # Output extracted text to stdout
        print(extracted_text)
        return 0
        
    except Exception as e:
        write_error(f"Error: EasyOCR text extraction failed: {e}")
        return 1

if __name__ == "__main__":
    sys.exit(main())
