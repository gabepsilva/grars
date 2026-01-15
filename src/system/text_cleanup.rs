//! Natural Reading API integration
//!
//! Sends text to a cloud-powered text enhancement service before TTS synthesis.
//! This service intelligently processes and refines text to improve speech quality.

use pulldown_cmark::{Event, Parser, Tag};
use tracing::{debug, info, warn};

const CLEANUP_API_URL: &str = "http://insight-reader-backend.i.psilva.org/api/content-cleanup";

/// Convert markdown to plain text by extracting only text content.
///
/// Strips all markdown formatting (bold, italic, headers, links, etc.)
/// and returns only the readable text content suitable for TTS.
/// Preserves line breaks to maintain natural pauses in speech.
fn markdown_to_plain_text(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut text_parts = Vec::new();

    for event in parser {
        match event {
            Event::Text(text) | Event::Code(text) => {
                // Preserve newlines within text (important for plain text like OCR output)
                // The text may contain newlines that need to be preserved
                text_parts.push(text.to_string());
            }
            Event::SoftBreak | Event::HardBreak => {
                // Line break - preserve as newline for a natural pause
                text_parts.push("\n".to_string());
            }
            Event::End(tag) => {
                // Block element end (paragraphs, headers, etc.) - add double newline for longer pause
                match tag {
                    Tag::Paragraph | Tag::Heading(..) | Tag::Item => {
                        text_parts.push("\n\n".to_string());
                    }
                    _ => {
                        // Other elements get a single newline if one isn't already present
                        if text_parts.last().map_or(true, |s| !s.ends_with('\n')) {
                            text_parts.push("\n".to_string());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Join parts and normalize: preserve newlines but normalize spaces within lines
    let result = text_parts.join("");

    // Split by newlines, trim and normalize spaces in each line, then rejoin
    result
        .lines()
        .map(|line| {
            // Normalize spaces within the line
            line.split_whitespace().collect::<Vec<_>>().join(" ")
        })
        .filter(|line| !line.is_empty()) // Remove empty lines
        .collect::<Vec<_>>()
        .join("\n")
}

/// Request body for the cleanup API.
#[derive(serde::Serialize)]
struct CleanupRequest<'a> {
    content: &'a str,
}

/// Response body from the cleanup API.
#[derive(serde::Deserialize)]
struct CleanupResponse {
    cleaned_content: String,
}

/// Send text to the Natural Reading API and return the enhanced text.
///
/// Makes a POST request to the cloud service with format: `{"content": text}`.
/// Returns the `cleaned_content` field from the JSON response, which contains
/// intelligently processed and refined text optimized for text-to-speech synthesis.
pub async fn cleanup_text(text: &str) -> Result<String, String> {
    info!(bytes = text.len(), "Sending text to Natural Reading service");
    debug!(text = %text, "Text being sent to Natural Reading service");

    let client = reqwest::Client::new();
    let request_body = CleanupRequest { content: text };

    let response = client
        .post(CLEANUP_API_URL)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to connect to Natural Reading service");
            format!("Failed to connect to Natural Reading service: {e}")
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        warn!(?status, body = %body, "Natural Reading service returned error");
        return Err(format!("Natural Reading service error ({}): {}", status, body));
    }

    let cleanup_response: CleanupResponse = response.json().await.map_err(|e| {
        warn!(error = %e, "Failed to parse Natural Reading service response");
        format!("Failed to parse Natural Reading service response: {e}")
    })?;

    // Log the text before markdown cleanup
    debug!(text = %cleanup_response.cleaned_content, "Text before markdown cleanup");

    // Check if the response looks like plain text (no markdown syntax)
    // If it's plain text, preserve newlines directly without markdown parsing
    let has_markdown_syntax = cleanup_response.cleaned_content.contains('#')
        || cleanup_response.cleaned_content.contains('*')
        || cleanup_response.cleaned_content.contains('[')
        || cleanup_response.cleaned_content.contains('`')
        || cleanup_response.cleaned_content.starts_with("```");

    let plain_text = if has_markdown_syntax {
        // Contains markdown - parse it
        markdown_to_plain_text(&cleanup_response.cleaned_content)
    } else {
        // Plain text - just normalize spaces within lines while preserving newlines
        cleanup_response.cleaned_content
            .lines()
            .map(|line| {
                // Normalize spaces within the line
                line.split_whitespace().collect::<Vec<_>>().join(" ")
            })
            .filter(|line| !line.is_empty()) // Remove empty lines
            .collect::<Vec<_>>()
            .join("\n")
    };

    info!(
        original_bytes = cleanup_response.cleaned_content.len(),
        plain_bytes = plain_text.len(),
        "Natural Reading completed, markdown stripped"
    );
    debug!(
        original_preview = %cleanup_response.cleaned_content.chars().take(100).collect::<String>(),
        plain_preview = %plain_text.chars().take(100).collect::<String>(),
        "Text preview (before and after markdown stripping)"
    );

    Ok(plain_text)
}

