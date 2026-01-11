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

var extractedTextParts: [String] = []
for observation in observations {
    let topCandidates = observation.topCandidates(1)
    guard let topCandidate = topCandidates.first else {
        continue
    }
    extractedTextParts.append(topCandidate.string)
}

let extractedText = extractedTextParts.joined(separator: " ")
if extractedText.isEmpty {
    exit(1)
}

print(extractedText)
