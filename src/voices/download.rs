//! Voice download functionality for Piper TTS
//!
//! Downloads voice model files (.onnx and .onnx.json) from Hugging Face.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use dirs::data_dir;
use tracing::{debug, info};

use crate::model::VoiceInfo;

const HUGGINGFACE_BASE_URL: &str = "https://huggingface.co/rhasspy/piper-voices/resolve/main";

/// Download a voice model from Hugging Face
///
/// Downloads both the .onnx and .onnx.json files to:
/// `~/.local/share/insight-reader/models/{voice_key}/`
pub async fn download_voice(
    voice_key: &str,
    voice_info: &VoiceInfo,
) -> Result<PathBuf, String> {
    info!(voice_key = %voice_key, "Starting voice download");
    
    // Determine model directory
    let model_dir = get_model_directory(voice_key)?;
    fs::create_dir_all(&model_dir)
        .map_err(|e| format!("Failed to create model directory: {e}"))?;
    
    // Find the .onnx and .onnx.json files in the voice info
    let onnx_file = voice_info
        .files
        .iter()
        .find(|(path, _)| path.ends_with(".onnx") && !path.ends_with(".onnx.json"))
        .ok_or_else(|| format!("No .onnx file found for voice {voice_key}"))?;
    
    let json_file = voice_info
        .files
        .iter()
        .find(|(path, _)| path.ends_with(".onnx.json"))
        .ok_or_else(|| format!("No .onnx.json file found for voice {voice_key}"))?;
    
    // Download .onnx file
    let onnx_url = format!("{}/{}", HUGGINGFACE_BASE_URL, onnx_file.0);
    let onnx_path = model_dir.join(format!("{}.onnx", voice_key));
    download_file(&onnx_url, &onnx_path, Some(&onnx_file.1.md5_digest)).await?;
    
    // Download .onnx.json file
    let json_url = format!("{}/{}", HUGGINGFACE_BASE_URL, json_file.0);
    let json_path = model_dir.join(format!("{}.onnx.json", voice_key));
    download_file(&json_url, &json_path, Some(&json_file.1.md5_digest)).await?;
    
    info!(voice_key = %voice_key, path = %model_dir.display(), "Voice download completed");
    Ok(model_dir.join(voice_key))
}

/// Download a single file from a URL
async fn download_file(
    url: &str,
    path: &Path,
    expected_md5: Option<&str>,
) -> Result<(), String> {
    debug!(url = %url, path = %path.display(), "Downloading file");
    
    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to fetch {url}: {e}"))?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to fetch {url}: HTTP {}", response.status()));
    }
    
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response body: {e}"))?;
    
    // Verify MD5 if provided
    if let Some(expected) = expected_md5 {
        let computed = format!("{:x}", md5::compute(&bytes));
        
        if computed != expected {
            return Err(format!(
                "MD5 checksum mismatch for {}: expected {}, got {}",
                path.display(),
                expected,
                computed
            ));
        }
        debug!(path = %path.display(), "MD5 checksum verified");
    }
    
    // Write file
    let mut file = fs::File::create(path)
        .map_err(|e| format!("Failed to create file {}: {e}", path.display()))?;
    file.write_all(&bytes)
        .map_err(|e| format!("Failed to write file {}: {e}", path.display()))?;
    
    debug!(path = %path.display(), bytes = bytes.len(), "File downloaded successfully");
    Ok(())
}

/// Get the model directory for a voice key
fn get_model_directory(_voice_key: &str) -> Result<PathBuf, String> {
    let data_dir = data_dir()
        .ok_or_else(|| "Failed to get data directory".to_string())?;
    
    Ok(data_dir.join("insight-reader").join("models"))
}

/// Check if a voice is already downloaded
pub fn is_voice_downloaded(voice_key: &str) -> bool {
    let model_dir = match get_model_directory(voice_key) {
        Ok(dir) => dir,
        Err(_) => return false,
    };
    
    let onnx_path = model_dir.join(format!("{}.onnx", voice_key));
    let json_path = model_dir.join(format!("{}.onnx.json", voice_key));
    
    onnx_path.exists() && json_path.exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_model_directory() {
        let result = get_model_directory("test_voice");
        assert!(result.is_ok());
    }
}
