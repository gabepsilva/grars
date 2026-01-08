//! Text cleanup API integration
//!
//! Sends text to a cleanup service before TTS synthesis.

use pulldown_cmark::{Event, Parser};
use tracing::{debug, info, warn};

const CLEANUP_API_URL: &str = "http://localhost:8080/api/prompt";

/// Convert markdown to plain text by extracting only text content.
///
/// Strips all markdown formatting (bold, italic, headers, links, etc.)
/// and returns only the readable text content suitable for TTS.
fn markdown_to_plain_text(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut text_parts = Vec::new();

    for event in parser {
        match event {
            Event::Text(text) | Event::Code(text) => {
                text_parts.push(text.to_string());
            }
            Event::SoftBreak | Event::HardBreak => {
                text_parts.push(" ".to_string());
            }
            Event::End(_) => {
                // Add space after block elements for readability
                if let Some(last) = text_parts.last_mut() {
                    if !last.ends_with(' ') {
                        text_parts.push(" ".to_string());
                    }
                }
            }
            _ => {}
        }
    }

    // Join and normalize whitespace (split_whitespace handles multiple spaces/newlines)
    text_parts
        .join("")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Request body for the cleanup API.
#[derive(serde::Serialize)]
struct CleanupRequest<'a> {
    prompt: &'a str,
}

/// Response body from the cleanup API.
#[derive(serde::Deserialize)]
struct CleanupResponse {
    response: String,
}

/// Send text to the cleanup API and return the cleaned text.
///
/// Makes a POST request to `http://localhost:8080/api/prompt` with `{"prompt": text}`.
/// Returns the `response` field from the JSON response.
pub async fn cleanup_text(text: &str) -> Result<String, String> {
    info!(bytes = text.len(), "Sending text to cleanup API");
    debug!(preview = %text.chars().take(100).collect::<String>(), "Text preview");

    let client = reqwest::Client::new();
    let request_body = CleanupRequest { prompt: text };

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

    // Strip markdown formatting from the response
    let plain_text = markdown_to_plain_text(&cleanup_response.response);

    info!(
        original_bytes = cleanup_response.response.len(),
        plain_bytes = plain_text.len(),
        "Text cleanup completed, markdown stripped"
    );
    debug!(
        original_preview = %cleanup_response.response.chars().take(100).collect::<String>(),
        plain_preview = %plain_text.chars().take(100).collect::<String>(),
        "Text preview (before and after markdown stripping)"
    );

    Ok(plain_text)
}

