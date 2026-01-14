#!/usr/bin/env swift

import Foundation
import Vision

func writeError(_ message: String) {
    let data = (message + "\n").data(using: .utf8)!
    FileHandle.standardError.write(data)
}

guard CommandLine.arguments.count == 2 else {
    writeError("Usage: extract_text_from_image.swift <image_path>")
    exit(1)
}

let imagePath = CommandLine.arguments[1]
guard FileManager.default.fileExists(atPath: imagePath) else {
    writeError("Error: Image file does not exist: \(imagePath)")
    exit(1)
}

guard let imageURL = URL(fileURLWithPath: imagePath) as URL?,
      let imageData = try? Data(contentsOf: imageURL) else {
    writeError("Error: Failed to load image")
    exit(1)
}

let requestHandler = VNImageRequestHandler(data: imageData, options: [:])
let textRequest = VNRecognizeTextRequest()
textRequest.recognitionLevel = .fast

// Configure recognition languages to support all available languages
// Supported languages as of macOS 13.0/iOS 16.0+:
// English, French, Italian, German, Spanish, Portuguese (Brazil),
// Chinese (Simplified/Traditional), Cantonese (Simplified/Traditional),
// Korean, Japanese, Russian, Ukrainian
// Note: To get the exact list at runtime, use:
// try textRequest.supportedRecognitionLanguages(for: .fast, revision: VNRecognizeTextRequestRevision1)
textRequest.recognitionLanguages = [
    "en-US",    // English
    "fr-FR",    // French
    "it-IT",    // Italian
    "de-DE",    // German
    "es-ES",    // Spanish
    "pt-BR",    // Portuguese
    "zh-Hans",  // Simplified Chinese
    "zh-Hant",  // Traditional Chinese
    "yue-Hans", // Simplified Cantonese
    "yue-Hant", // Traditional Cantonese
    "ko-KR",    // Korean
    "ja-JP",    // Japanese
    "ru-RU",    // Russian
    "uk-UA",    // Ukrainian
]

// Enable automatic language detection as fallback
// This helps when the text contains multiple languages or languages not in the list above
textRequest.automaticallyDetectsLanguage = true

do {
    try requestHandler.perform([textRequest])
} catch {
    writeError("Error: Vision framework request failed: \(error)")
    exit(1)
}

guard let observations = textRequest.results, !observations.isEmpty else {
    // No text found - exit with code 1 but no error message (this is expected)
    exit(1)
}

// Group observations by Y-coordinate to preserve line breaks
// Observations with similar Y-coordinates are on the same line
var lineGroups: [[VNRecognizedTextObservation]] = []
var currentLine: [VNRecognizedTextObservation] = []
var lastY: CGFloat? = nil
let yTolerance: CGFloat = 10.0 // Pixels - text within this Y range is considered same line

// Sort observations by Y-coordinate (top to bottom)
let sortedObservations = observations.sorted { obs1, obs2 in
    let y1 = obs1.boundingBox.midY
    let y2 = obs2.boundingBox.midY
    if abs(y1 - y2) < yTolerance {
        // Same line, sort by X (left to right)
        return obs1.boundingBox.minX < obs2.boundingBox.minX
    }
    return y1 < y2
}

for observation in sortedObservations {
    let currentY = observation.boundingBox.midY
    
    if let lastYValue = lastY {
        // Check if this observation is on a new line
        if abs(currentY - lastYValue) > yTolerance {
            // New line detected
            if !currentLine.isEmpty {
                lineGroups.append(currentLine)
                currentLine = []
            }
        }
    }
    
    currentLine.append(observation)
    lastY = currentY
}

// Add the last line
if !currentLine.isEmpty {
    lineGroups.append(currentLine)
}

// Extract text from each line group
var extractedLines: [String] = []
for lineGroup in lineGroups {
    var lineTextParts: [String] = []
    for observation in lineGroup {
        let topCandidates = observation.topCandidates(1)
        guard let topCandidate = topCandidates.first else {
            continue
        }
        lineTextParts.append(topCandidate.string)
    }
    if !lineTextParts.isEmpty {
        // Join words on the same line with spaces
        extractedLines.append(lineTextParts.joined(separator: " "))
    }
}

// Join lines with newlines to preserve line breaks
let extractedText = extractedLines.joined(separator: "\n")
if extractedText.isEmpty {
    exit(1)
}

print(extractedText)
