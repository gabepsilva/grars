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
        // Validate it's a known country code
        if is_known_country(country) {
            return match country {
                "BR" => "BR",
                "PT" => "PT",
                "US" => "US",
                "GB" => "GB",
                "AU" => "AU",
                "CA" => "CA",
                "ES" => "ES",
                "MX" => "MX",
                "AR" => "AR",
                "CO" => "CO",
                "FR" => "FR",
                "DE" => "DE",
                "AT" => "AT",
                "CH" => "CH",
                "IT" => "IT",
                "NL" => "NL",
                "PL" => "PL",
                "RU" => "RU",
                "TR" => "TR",
                "GR" => "GR",
                "CZ" => "CZ",
                "SK" => "SK",
                "HU" => "HU",
                "RO" => "RO",
                "BG" => "BG",
                "HR" => "HR",
                "SI" => "SI",
                "FI" => "FI",
                "SE" => "SE",
                "NO" => "NO",
                "DK" => "DK",
                "IS" => "IS",
                "EE" => "EE",
                "LV" => "LV",
                "LT" => "LT",
                "CN" => "CN",
                "TW" => "TW",
                "HK" => "HK",
                "JP" => "JP",
                "KR" => "KR",
                "VN" => "VN",
                "TH" => "TH",
                "ID" => "ID",
                "MY" => "MY",
                "PH" => "PH",
                "IN" => "IN",
                "PK" => "PK",
                "BD" => "BD",
                "SA" => "SA",
                "AE" => "AE",
                "IL" => "IL",
                "IR" => "IR",
                "IQ" => "IQ",
                "JO" => "JO",
                "ZA" => "ZA",
                "EG" => "EG",
                "KE" => "KE",
                "NG" => "NG",
                "CL" => "CL",
                "PE" => "PE",
                "VE" => "VE",
                "EC" => "EC",
                "BO" => "BO",
                "PY" => "PY",
                "UY" => "UY",
                "CR" => "CR",
                "PA" => "PA",
                "DO" => "DO",
                "CU" => "CU",
                _ => "GLOBE",
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
