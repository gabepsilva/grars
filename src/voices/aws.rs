//! AWS Polly voice metadata management
//!
//! Handles fetching and organizing voices from AWS Polly using the AWS SDK.

use std::collections::HashMap;
use tracing::{debug, trace};

use crate::model::LanguageInfo;

/// Voice metadata from AWS Polly
#[derive(Debug, Clone)]
pub struct PollyVoiceInfo {
    pub id: String,              // AWS VoiceId (e.g., "Matthew", "Joanna")
    pub name: String,            // Voice name
    pub language: LanguageInfo,  // Language information
    pub gender: String,          // "Male" or "Female"
    pub engine: String,         // "standard" or "neural"
}

/// Fetch voices from AWS Polly using the AWS SDK
pub async fn fetch_polly_voices() -> Result<HashMap<String, PollyVoiceInfo>, String> {
    debug!("AWS Polly: starting fetch_polly_voices");

    // Determine region: check ~/.aws/config, env vars, or default to us-east-1
    let region = detect_aws_region();
    debug!(region = %region, "AWS Polly: using region for voice fetching");

    // Load AWS config (credentials from ~/.aws/credentials or env vars)
    // This is async and will use the existing tokio runtime from Iced
    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(aws_config::Region::new(region.clone()))
        .load()
        .await;

    let client = aws_sdk_polly::Client::new(&config);
    debug!("AWS Polly: client created for voice fetching");

    // Fetch all voices from AWS Polly
    debug!("AWS Polly: calling DescribeVoices without filters");
    let response = client
        .describe_voices()
        .send()
        .await
        .map_err(|e| format!("Failed to fetch voices from AWS Polly: {e}"))?;

    let aws_voices = response.voices();
    let voices_vec: Vec<aws_sdk_polly::types::Voice> = aws_voices.iter().cloned().collect();
    debug!(
        count = voices_vec.len(),
        "AWS Polly: received voices from DescribeVoices"
    );

    // Convert AWS voices to our internal format
    // Create separate entries for each engine type a voice supports
    let mut voices: HashMap<String, PollyVoiceInfo> = HashMap::new();

    for voice in voices_vec {
        let voice_id = voice.id().map(|v| v.as_str().to_string());
        let name = voice.name().map(|n| n.to_string());
        let language_code = voice.language_code().map(|l| l.as_str().to_string());
        let gender = voice.gender().map(|g| format!("{:?}", g));
        let supported_engines = voice.supported_engines();

        trace!(
            id = voice_id.as_deref().unwrap_or("<none>"),
            name = name.as_deref().unwrap_or("<none>"),
            language_code = language_code.as_deref().unwrap_or("<none>"),
            gender = gender.as_deref().unwrap_or("<none>"),
            engines = ?supported_engines,
            "AWS Polly: raw voice from DescribeVoices"
        );

        if let (Some(id), Some(lang_code)) = (voice_id, language_code) {
            // Create language info from AWS language code
            let language_info = create_language_info(&lang_code);

            // Create a separate entry for each supported engine
            // Use format "VoiceId:Engine" as the key to distinguish engine variants
            for engine in supported_engines {
                let engine_str = format!("{:?}", engine);
                let key = format!("{}:{}", id, engine_str);
                
                let voice_info = PollyVoiceInfo {
                    id: id.clone(),
                    name: name.clone().unwrap_or_else(|| id.clone()),
                    language: language_info.clone(),
                    gender: gender.clone().unwrap_or_else(|| "Unknown".to_string()),
                    engine: engine_str,
                };

                voices.insert(key, voice_info);
            }
        }
    }

    debug!(
        count = voices.len(),
        "AWS Polly: converted AWS voices to internal format"
    );
    Ok(voices)
}

/// Detect AWS region from environment or config file.
///
/// Priority:
/// 1. AWS_REGION or AWS_DEFAULT_REGION environment variables
/// 2. ~/.aws/config file (default profile)
/// 3. Falls back to us-east-1
pub fn detect_aws_region() -> String {
    // Check environment variables first
    if let Ok(region) = std::env::var("AWS_REGION") {
        if !region.is_empty() {
            return region;
        }
    }
    if let Ok(region) = std::env::var("AWS_DEFAULT_REGION") {
        if !region.is_empty() {
            return region;
        }
    }

    // Check ~/.aws/config file
    if let Some(home) = dirs::home_dir() {
        let config_path = home.join(".aws").join("config");
        if let Some(region) = read_region_from_config(&config_path) {
            return region;
        }
    }

    // Default to us-east-1
    "us-east-1".to_string()
}

/// Read region from AWS config file.
pub(crate) fn read_region_from_config(path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let profile = std::env::var("AWS_PROFILE").unwrap_or_else(|_| "default".to_string());

    // Look for [default] or [profile <name>] section
    let section_header = if profile == "default" {
        "[default]"
    } else {
        return read_region_from_profile_section(&content, &profile);
    };

    let mut in_section = false;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_section = line.eq_ignore_ascii_case(section_header);
            continue;
        }
        if in_section && line.starts_with("region") {
            if let Some(value) = line.split('=').nth(1) {
                let region = value.trim();
                if !region.is_empty() {
                    return Some(region.to_string());
                }
            }
        }
    }
    None
}

/// Read region from a named profile section.
pub(crate) fn read_region_from_profile_section(content: &str, profile: &str) -> Option<String> {
    let section_header = format!("[profile {}]", profile);
    let mut in_section = false;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_section = line.eq_ignore_ascii_case(&section_header);
            continue;
        }
        if in_section && line.starts_with("region") {
            if let Some(value) = line.split('=').nth(1) {
                let region = value.trim();
                if !region.is_empty() {
                    return Some(region.to_string());
                }
            }
        }
    }
    None
}

/// Create LanguageInfo from AWS language code
fn create_language_info(lang_code: &str) -> LanguageInfo {
    // AWS language codes are typically in format like "en-US", "pt-BR", etc.
    // We need to convert to our format like "en_US", "pt_BR"
    let normalized_code = lang_code.replace('-', "_");
    
    // Extract language and region
    let parts: Vec<&str> = normalized_code.split('_').collect();
    let lang_family = parts.first().copied().unwrap_or("en").to_string();
    let region = parts.get(1).copied().unwrap_or("US").to_string();

    // Map common language codes to language names
    let (name_english, country_english) = get_language_names(&lang_family, &region);
    let name_native = name_english.clone(); // AWS doesn't provide native names

    LanguageInfo {
        code: normalized_code,
        family: lang_family,
        region,
        name_native,
        name_english,
        country_english,
    }
}

/// Get language and country names from language code
fn get_language_names(lang_family: &str, region: &str) -> (String, String) {
    // Map of language families to English names
    let lang_name = match lang_family {
        "en" => "English",
        "pt" => "Portuguese",
        "es" => "Spanish",
        "fr" => "French",
        "de" => "German",
        "it" => "Italian",
        "ja" => "Japanese",
        "ko" => "Korean",
        "zh" => "Chinese",
        "ar" => "Arabic",
        "hi" => "Hindi",
        "ru" => "Russian",
        "nl" => "Dutch",
        "pl" => "Polish",
        "tr" => "Turkish",
        "sv" => "Swedish",
        "da" => "Danish",
        "no" => "Norwegian",
        "fi" => "Finnish",
        "cs" => "Czech",
        "ro" => "Romanian",
        "hu" => "Hungarian",
        "th" => "Thai",
        "vi" => "Vietnamese",
        "id" => "Indonesian",
        "ms" => "Malay",
        "he" => "Hebrew",
        "is" => "Icelandic",
        "cy" => "Welsh",
        "ga" => "Irish",
        "mt" => "Maltese",
        _ => "Unknown",
    };

    // Map of regions to country names
    let country_name = match region {
        "US" => "United States",
        "GB" => "United Kingdom",
        "AU" => "Australia",
        "CA" => "Canada",
        "IN" => "India",
        "IE" => "Ireland",
        "NZ" => "New Zealand",
        "ZA" => "South Africa",
        "BR" => "Brazil",
        "PT" => "Portugal",
        "ES" => "Spain",
        "MX" => "Mexico",
        "AR" => "Argentina",
        "CO" => "Colombia",
        "CL" => "Chile",
        "PE" => "Peru",
        "VE" => "Venezuela",
        "EC" => "Ecuador",
        "BO" => "Bolivia",
        "PY" => "Paraguay",
        "UY" => "Uruguay",
        "CR" => "Costa Rica",
        "PA" => "Panama",
        "DO" => "Dominican Republic",
        "CU" => "Cuba",
        "FR" => "France",
        "DE" => "Germany",
        "AT" => "Austria",
        "CH" => "Switzerland",
        "IT" => "Italy",
        "NL" => "Netherlands",
        "PL" => "Poland",
        "RU" => "Russia",
        "TR" => "Turkey",
        "GR" => "Greece",
        "CZ" => "Czech Republic",
        "SK" => "Slovakia",
        "HU" => "Hungary",
        "RO" => "Romania",
        "BG" => "Bulgaria",
        "HR" => "Croatia",
        "SI" => "Slovenia",
        "FI" => "Finland",
        "SV" => "Sweden",
        "NO" => "Norway",
        "DK" => "Denmark",
        "IS" => "Iceland",
        "EE" => "Estonia",
        "LV" => "Latvia",
        "LT" => "Lithuania",
        "CN" => "China",
        "TW" => "Taiwan",
        "HK" => "Hong Kong",
        "JP" => "Japan",
        "KR" => "South Korea",
        "VN" => "Vietnam",
        "TH" => "Thailand",
        "ID" => "Indonesia",
        "MY" => "Malaysia",
        "PH" => "Philippines",
        "PK" => "Pakistan",
        "BD" => "Bangladesh",
        "SA" => "Saudi Arabia",
        "AE" => "United Arab Emirates",
        "IL" => "Israel",
        "IR" => "Iran",
        "IQ" => "Iraq",
        "JO" => "Jordan",
        "EG" => "Egypt",
        "KE" => "Kenya",
        "NG" => "Nigeria",
        _ => "Unknown",
    };

    (lang_name.to_string(), country_name.to_string())
}

/// Get unique language codes from AWS voices
pub fn get_available_languages(
    voices: &HashMap<String, PollyVoiceInfo>,
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

    debug!(count = lang_list.len(), "Extracted available languages from AWS voices");
    lang_list
}

/// Get voices for a specific language code
pub fn get_voices_for_language<'a>(
    voices: &'a HashMap<String, PollyVoiceInfo>,
    language_code: &'a str,
) -> Vec<&'a PollyVoiceInfo> {
    voices
        .values()
        .filter(|voice| voice.language.code == language_code)
        .collect()
}
