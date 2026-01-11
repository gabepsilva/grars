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
textRequest.recognitionLevel = .accurate

// Configure recognition languages to support Chinese and other common languages
// This significantly improves accuracy for non-Latin scripts like Chinese
textRequest.recognitionLanguages = [
    "zh-Hans",  // Simplified Chinese
    "zh-Hant",  // Traditional Chinese
    "en-US",    // English
    "ja-JP",    // Japanese
    "ko-KR",    // Korean
    "fr-FR",    // French
    "de-DE",    // German
    "es-ES",    // Spanish
    "it-IT",    // Italian
    "pt-BR",    // Portuguese
    "ru-RU",    // Russian
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
