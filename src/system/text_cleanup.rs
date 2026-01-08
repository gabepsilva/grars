//! Text cleanup API integration
//!
//! Sends text to a cleanup service before TTS synthesis.

use tracing::{debug, info, warn};

const CLEANUP_API_URL: &str = "http://localhost:8080/api/prompt";

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

    info!(
        bytes = cleanup_response.response.len(),
        "Text cleanup completed successfully"
    );
    debug!(
        preview = %cleanup_response.response.chars().take(100).collect::<String>(),
        "Cleaned text preview"
    );

    Ok(cleanup_response.response)
}

