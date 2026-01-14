//! Language code to country code mapping.
//!
//! Maps language codes (e.g., "pt_BR", "en_US") to ISO 3166-1 alpha-2
//! country codes for flag lookup.

/// Map a language code to its corresponding country code.
///
/// Handles two formats:
/// - Full format: "lang_COUNTRY" (e.g., "pt_BR" -> "BR")
/// - Language only: "lang" (e.g., "ja" -> "JP")
///
/// Returns "GLOBE" for unknown languages (shows globe fallback icon).
pub fn lang_to_country(lang_code: &str) -> &'static str {
    // First, try to extract country from "lang_COUNTRY" format
    if let Some(country) = lang_code.split('_').nth(1) {
        // If it's a known country code, return the static string literal
        // We need to match to get a 'static lifetime, not just return the slice
        if is_known_country(country) {
            // Convert to uppercase and match to return static string
            return match country.to_uppercase().as_str() {
                "AR" => "AR", "BO" => "BO", "BR" => "BR", "CA" => "CA", "CL" => "CL",
                "CO" => "CO", "CR" => "CR", "CU" => "CU", "DO" => "DO", "EC" => "EC",
                "MX" => "MX", "PA" => "PA", "PE" => "PE", "PY" => "PY", "US" => "US",
                "UY" => "UY", "VE" => "VE", "AL" => "AL", "AT" => "AT", "BG" => "BG",
                "BY" => "BY", "CH" => "CH", "CZ" => "CZ", "DE" => "DE", "DK" => "DK",
                "EE" => "EE", "ES" => "ES", "FI" => "FI", "FR" => "FR", "GB" => "GB",
                "GR" => "GR", "HR" => "HR", "HU" => "HU", "IE" => "IE", "IS" => "IS",
                "IT" => "IT", "LT" => "LT", "LV" => "LV", "MK" => "MK", "MT" => "MT",
                "NL" => "NL", "NO" => "NO", "PL" => "PL", "PT" => "PT", "RO" => "RO",
                "RS" => "RS", "RU" => "RU", "SE" => "SE", "SI" => "SI", "SK" => "SK",
                "TR" => "TR", "UA" => "UA", "AZ" => "AZ", "BD" => "BD", "CN" => "CN",
                "GE" => "GE", "HK" => "HK", "ID" => "ID", "IL" => "IL", "IN" => "IN",
                "IQ" => "IQ", "IR" => "IR", "JO" => "JO", "JP" => "JP", "KG" => "KG",
                "KH" => "KH", "KR" => "KR", "KZ" => "KZ", "LA" => "LA", "LK" => "LK",
                "MM" => "MM", "MN" => "MN", "MY" => "MY", "NP" => "NP", "PK" => "PK",
                "PH" => "PH", "SA" => "SA", "TH" => "TH", "TW" => "TW", "UZ" => "UZ",
                "VN" => "VN", "AE" => "AE", "EG" => "EG", "ER" => "ER", "ET" => "ET",
                "GH" => "GH", "KE" => "KE", "MG" => "MG", "ML" => "ML", "MW" => "MW",
                "NG" => "NG", "RW" => "RW", "SN" => "SN", "SO" => "SO", "UG" => "UG",
                "ZA" => "ZA", "ZW" => "ZW", "AM" => "AM",
                _ => "GLOBE", // Shouldn't happen since we checked is_known_country, but safe fallback
            };
        }
    }

    // Fallback: map language family to primary country
    let lang = lang_code.split('_').next().unwrap_or("");
    match lang {
        // Middle Eastern / Arabic
        "ar" => "SA", // Arabic -> Saudi Arabia
        "he" => "IL", // Hebrew -> Israel
        "fa" => "IR", // Persian -> Iran

        // East Asian
        "zh" => "CN", // Chinese -> China
        "ja" => "JP", // Japanese -> Japan
        "ko" => "KR", // Korean -> South Korea
        "vi" => "VN", // Vietnamese -> Vietnam
        "th" => "TH", // Thai -> Thailand
        "km" => "KH", // Khmer -> Cambodia
        "lo" => "LA", // Lao -> Laos
        "my" => "MM", // Burmese -> Myanmar
        "mn" => "MN", // Mongolian -> Mongolia

        // South Asian
        "hi" => "IN", // Hindi -> India
        "ur" => "PK", // Urdu -> Pakistan
        "bn" => "BD", // Bengali -> Bangladesh
        "ta" => "IN", // Tamil -> India
        "te" => "IN", // Telugu -> India
        "ml" => "IN", // Malayalam -> India
        "kn" => "IN", // Kannada -> India
        "gu" => "IN", // Gujarati -> India
        "pa" => "IN", // Punjabi -> India
        "mr" => "IN", // Marathi -> India
        "ne" => "NP", // Nepali -> Nepal
        "si" => "LK", // Sinhala -> Sri Lanka

        // Central European
        "cs" => "CZ", // Czech -> Czech Republic
        "sk" => "SK", // Slovak -> Slovakia
        "hu" => "HU", // Hungarian -> Hungary
        "ro" => "RO", // Romanian -> Romania
        "bg" => "BG", // Bulgarian -> Bulgaria
        "hr" => "HR", // Croatian -> Croatia
        "sr" => "RS", // Serbian -> Serbia
        "sl" => "SI", // Slovenian -> Slovenia

        // Baltic
        "et" => "EE", // Estonian -> Estonia
        "lv" => "LV", // Latvian -> Latvia
        "lt" => "LT", // Lithuanian -> Lithuania

        // Nordic
        "fi" => "FI", // Finnish -> Finland
        "sv" => "SE", // Swedish -> Sweden
        "no" => "NO", // Norwegian -> Norway
        "da" => "DK", // Danish -> Denmark
        "is" => "IS", // Icelandic -> Iceland

        // Iberian
        "ca" => "ES", // Catalan -> Spain
        "eu" => "ES", // Basque -> Spain
        "gl" => "ES", // Galician -> Spain

        // Eastern European / Slavic
        "uk" => "UA", // Ukrainian -> Ukraine
        "be" => "BY", // Belarusian -> Belarus
        "mk" => "MK", // Macedonian -> North Macedonia
        "sq" => "AL", // Albanian -> Albania

        // Other European
        "mt" => "MT", // Maltese -> Malta
        "ga" => "IE", // Irish -> Ireland
        "cy" => "GB", // Welsh -> UK

        // Caucasus / Central Asian
        "ka" => "GE", // Georgian -> Georgia
        "hy" => "AM", // Armenian -> Armenia
        "az" => "AZ", // Azerbaijani -> Azerbaijan
        "kk" => "KZ", // Kazakh -> Kazakhstan
        "ky" => "KG", // Kyrgyz -> Kyrgyzstan
        "uz" => "UZ", // Uzbek -> Uzbekistan

        // African
        "sw" => "KE", // Swahili -> Kenya
        "af" => "ZA", // Afrikaans -> South Africa
        "am" => "ET", // Amharic -> Ethiopia
        "yo" => "NG", // Yoruba -> Nigeria
        "ig" => "NG", // Igbo -> Nigeria
        "ha" => "NG", // Hausa -> Nigeria
        "zu" => "ZA", // Zulu -> South Africa
        "xh" => "ZA", // Xhosa -> South Africa
        "st" => "ZA", // Southern Sotho -> South Africa
        "tn" => "ZA", // Tswana -> South Africa
        "sn" => "ZW", // Shona -> Zimbabwe
        "ny" => "MW", // Chichewa -> Malawi
        "so" => "SO", // Somali -> Somalia
        "om" => "ET", // Oromo -> Ethiopia
        "ti" => "ER", // Tigrinya -> Eritrea
        "mg" => "MG", // Malagasy -> Madagascar
        "rw" => "RW", // Kinyarwanda -> Rwanda
        "lg" => "UG", // Ganda -> Uganda
        "ak" => "GH", // Akan -> Ghana
        "ff" => "SN", // Fulah -> Senegal
        "wo" => "SN", // Wolof -> Senegal
        "bm" => "ML", // Bambara -> Mali
        "ee" => "GH", // Ewe -> Ghana
        "tw" => "GH", // Twi -> Ghana

        // Default fallback
        _ => "GLOBE",
    }
}

/// Check if a country code is known/supported.
fn is_known_country(code: &str) -> bool {
    matches!(
        code,
        "AR" | "BO"
            | "BR"
            | "CA"
            | "CL"
            | "CO"
            | "CR"
            | "CU"
            | "DO"
            | "EC"
            | "MX"
            | "PA"
            | "PE"
            | "PY"
            | "US"
            | "UY"
            | "VE"
            | "AL"
            | "AT"
            | "BG"
            | "BY"
            | "CH"
            | "CZ"
            | "DE"
            | "DK"
            | "EE"
            | "ES"
            | "FI"
            | "FR"
            | "GB"
            | "GR"
            | "HR"
            | "HU"
            | "IE"
            | "IS"
            | "IT"
            | "LT"
            | "LV"
            | "MK"
            | "MT"
            | "NL"
            | "NO"
            | "PL"
            | "PT"
            | "RO"
            | "RS"
            | "RU"
            | "SE"
            | "SI"
            | "SK"
            | "TR"
            | "UA"
            | "AZ"
            | "BD"
            | "CN"
            | "GE"
            | "HK"
            | "ID"
            | "IL"
            | "IN"
            | "IQ"
            | "IR"
            | "JO"
            | "JP"
            | "KG"
            | "KH"
            | "KR"
            | "KZ"
            | "LA"
            | "LK"
            | "MM"
            | "MN"
            | "MY"
            | "NP"
            | "PK"
            | "PH"
            | "SA"
            | "TH"
            | "TW"
            | "UZ"
            | "VN"
            | "AE"
            | "EG"
            | "ER"
            | "ET"
            | "GH"
            | "KE"
            | "MG"
            | "ML"
            | "MW"
            | "NG"
            | "RW"
            | "SN"
            | "SO"
            | "UG"
            | "ZA"
            | "ZW"
            | "AM"
    )
}
