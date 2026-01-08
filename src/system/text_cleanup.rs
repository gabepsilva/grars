//! Text cleanup API integration
//!
//! Sends text to a cleanup service before TTS synthesis.

use pulldown_cmark::{Event, Parser, Tag};
use tracing::{debug, info, warn};

const CLEANUP_API_URL: &str = "http://localhost:8080/api/content-cleanup";

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

/// Send text to the cleanup API and return the cleaned text.
///
/// Makes a POST request to `http://localhost:8080/api/content-cleanup` with
/// format: `{"content": text}`.
/// Returns the `cleaned_content` field from the JSON response.
pub async fn cleanup_text(text: &str) -> Result<String, String> {
    info!(bytes = text.len(), "Sending text to cleanup API");
    debug!(text = %text, "Text being sent to cleanup API");

    let client = reqwest::Client::new();
    let request_body = CleanupRequest { content: text };

    let response = client
        .post(CLEANUP_API_URL)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to connect to cleanup API");
            format!("Failed to connect to cleanup API: {e}")
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        warn!(?status, body = %body, "Cleanup API returned error");
        return Err(format!("Cleanup API error ({}): {}", status, body));
    }

    let cleanup_response: CleanupResponse = response.json().await.map_err(|e| {
        warn!(error = %e, "Failed to parse cleanup API response");
        format!("Failed to parse cleanup API response: {e}")
    })?;

    // Log the text before markdown cleanup
    debug!(text = %cleanup_response.cleaned_content, "Text before markdown cleanup");

    // Strip markdown formatting from the response
    let plain_text = markdown_to_plain_text(&cleanup_response.cleaned_content);

    info!(
        original_bytes = cleanup_response.cleaned_content.len(),
        plain_bytes = plain_text.len(),
        "Text cleanup completed, markdown stripped"
    );
    debug!(
        original_preview = %cleanup_response.cleaned_content.chars().take(100).collect::<String>(),
        plain_preview = %plain_text.chars().take(100).collect::<String>(),
        "Text preview (before and after markdown stripping)"
    );

    Ok(plain_text)
}

