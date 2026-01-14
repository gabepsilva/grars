#!/usr/bin/env python3
"""
Extract text from an image using EasyOCR.
Similar to install/extract_text_from_image.swift for macOS, but uses EasyOCR instead of Vision framework.
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
        
        # Group text by Y-coordinate to preserve line breaks
        # results is a list of tuples: (bbox, text, confidence)
        # bbox is a list of 4 points: [[x1, y1], [x2, y2], [x3, y3], [x4, y4]]
        # We'll use the average Y-coordinate to group into lines
        y_tolerance = 10.0  # Pixels - text within this Y range is considered same line
        
        # Sort results by Y-coordinate (top to bottom)
        def get_y_center(bbox):
            """Get the center Y-coordinate of a bounding box."""
            y_coords = [point[1] for point in bbox]
            return sum(y_coords) / len(y_coords)
        
        def get_x_min(bbox):
            """Get the minimum X-coordinate for sorting within a line."""
            x_coords = [point[0] for point in bbox]
            return min(x_coords)
        
        sorted_results = sorted(results, key=lambda r: (get_y_center(r[0]), get_x_min(r[0])))
        
        # Group into lines
        line_groups = []
        current_line = []
        last_y = None
        
        for (bbox, text, confidence) in sorted_results:
            current_y = get_y_center(bbox)
            
            if last_y is not None:
                # Check if this text is on a new line
                if abs(current_y - last_y) > y_tolerance:
                    # New line detected
                    if current_line:
                        line_groups.append(current_line)
                        current_line = []
            
            current_line.append((bbox, text, confidence))
            last_y = current_y
        
        # Add the last line
        if current_line:
            line_groups.append(current_line)
        
        # Extract text from each line group
        extracted_lines = []
        for line_group in line_groups:
            line_text_parts = [text for (_, text, _) in line_group]
            if line_text_parts:
                # Join words on the same line with spaces
                extracted_lines.append(" ".join(line_text_parts))
        
        # Join lines with newlines to preserve line breaks
        extracted_text = "\n".join(extracted_lines)
        
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
