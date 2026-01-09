//! Voice metadata management for Piper TTS
//!
//! Handles fetching and parsing voices.json from Hugging Face's piper-voices repository.

pub mod download;

use std::collections::HashMap;
use tracing::debug;

use crate::model::{LanguageInfo, VoiceInfo};

const VOICES_JSON_URL: &str = "https://huggingface.co/rhasspy/piper-voices/resolve/main/voices.json";

/// Fetch voices.json from Hugging Face
pub async fn fetch_voices_json() -> Result<HashMap<String, VoiceInfo>, String> {
    debug!("Fetching voices.json from Hugging Face");
    
    let response = reqwest::get(VOICES_JSON_URL)
        .await
        .map_err(|e| format!("Failed to fetch voices.json: {e}"))?;
    
    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch voices.json: HTTP {}",
            response.status()
        ));
    }
    
    let json_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {e}"))?;
    
    debug!(bytes = json_text.len(), "Received voices.json");
    
    parse_voices_json(&json_text)
}

/// Parse voices.json into a HashMap of VoiceInfo
pub fn parse_voices_json(json_text: &str) -> Result<HashMap<String, VoiceInfo>, String> {
    let voices: HashMap<String, VoiceInfo> = serde_json::from_str(json_text)
        .map_err(|e| format!("Failed to parse voices.json: {e}"))?;
    
    debug!(count = voices.len(), "Parsed voices.json");
    Ok(voices)
}

/// Get unique language codes from voices
pub fn get_available_languages(
    voices: &HashMap<String, VoiceInfo>,
) -> Vec<(String, LanguageInfo)> {
    let mut languages: HashMap<String, LanguageInfo> = HashMap::new();
    
    for voice in voices.values() {
        let lang_code = &voice.language.code;
        if !languages.contains_key(lang_code) {
            languages.insert(lang_code.clone(), voice.language.clone());
        }
    }
    
    let mut lang_list: Vec<(String, LanguageInfo)> = languages.into_iter().collect();
    // Sort by language code for consistent display
    lang_list.sort_by_key(|(code, _)| code.clone());
    
    debug!(count = lang_list.len(), "Extracted available languages");
    lang_list
}

/// Get voices for a specific language code
pub fn get_voices_for_language<'a>(
    voices: &'a HashMap<String, VoiceInfo>,
    language_code: &'a str,
) -> Vec<&'a VoiceInfo> {
    voices
        .values()
        .filter(|voice| voice.language.code == language_code)
        .collect()
}
