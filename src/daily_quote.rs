use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

const API_URL: &str = "https://open.iciba.com/dsapi/";
const CACHE_FILE: &str = "daily-quote.json";
const IMAGE_FILE: &str = "daily-quote.png";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DailyQuote {
    pub content: String,
    pub note: String,
    pub dateline: String,
    pub picture_url: Option<String>,
    pub local_image_path: Option<String>,
}

#[derive(Deserialize)]
struct IcibaResponse {
    content: String,
    #[serde(default)]
    note: String,
    #[serde(default)]
    dateline: String,
    #[serde(default)]
    picture: String,
}

pub fn parse_iciba_response(json: &str) -> Result<DailyQuote, Box<dyn Error + Send + Sync>> {
    let response: IcibaResponse = serde_json::from_str(json)?;
    if response.content.trim().is_empty() {
        return Err("ICIBA response contains no quote".into());
    }

    Ok(DailyQuote {
        content: response.content.trim().to_string(),
        note: response.note.trim().to_string(),
        dateline: response.dateline.trim().to_string(),
        picture_url: non_empty(response.picture),
        local_image_path: None,
    })
}

pub fn fallback_quote() -> DailyQuote {
    DailyQuote {
        content: "Small steps still move you forward.".to_string(),
        note: "每一个微小的行动，都在让你靠近目标。".to_string(),
        dateline: String::new(),
        picture_url: None,
        local_image_path: None,
    }
}

pub fn default_cache_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cache")
        .join("raspberry-clock")
}

pub fn load_cached(cache_dir: &Path) -> Option<DailyQuote> {
    let content = fs::read_to_string(cache_dir.join(CACHE_FILE)).ok()?;
    let quote: DailyQuote = serde_json::from_str(&content).ok()?;
    (!quote.content.trim().is_empty()).then_some(quote)
}

pub fn fetch_and_cache(cache_dir: &Path) -> Result<DailyQuote, Box<dyn Error + Send + Sync>> {
    fs::create_dir_all(cache_dir)?;
    let previous = load_cached(cache_dir);
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(8))
        .timeout_read(Duration::from_secs(12))
        .build();

    let response = agent.get(API_URL).call()?;
    let json = response.into_string()?;
    let mut quote = parse_iciba_response(&json)?;

    if let Some(picture_url) = quote.picture_url.as_deref() {
        match download_image(&agent, picture_url, &cache_dir.join(IMAGE_FILE)) {
            Ok(path) => quote.local_image_path = Some(path.to_string_lossy().into_owned()),
            Err(error) => {
                eprintln!("daily quote image download failed: {error}");
                quote.local_image_path = previous
                    .as_ref()
                    .and_then(|cached| cached.local_image_path.clone())
                    .filter(|path| Path::new(path).exists());
            }
        }
    } else {
        quote.local_image_path = previous
            .as_ref()
            .and_then(|cached| cached.local_image_path.clone())
            .filter(|path| Path::new(path).exists());
    }

    write_json_atomic(&cache_dir.join(CACHE_FILE), &quote)?;
    Ok(quote)
}

fn download_image(
    agent: &ureq::Agent,
    url: &str,
    destination: &Path,
) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    let response = agent.get(url).call()?;
    let mut bytes = Vec::new();
    response
        .into_reader()
        .take(8 * 1024 * 1024)
        .read_to_end(&mut bytes)?;
    if bytes.is_empty() {
        return Err("daily quote image is empty".into());
    }

    write_bytes_atomic(destination, &bytes)?;
    Ok(destination.to_path_buf())
}

fn write_json_atomic(
    destination: &Path,
    quote: &DailyQuote,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let json = serde_json::to_vec(quote)?;
    write_bytes_atomic(destination, &json)
}

fn write_bytes_atomic(
    destination: &Path,
    bytes: &[u8],
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let temporary = destination.with_extension("tmp");
    let mut file = fs::File::create(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(temporary, destination)?;
    Ok(())
}

fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "sid":"5996",
        "content":"Silence grows in the forest.",
        "note":"寂静在树林里生长。",
        "picture":"https://staticedu-wps-cache.iciba.com/image/example.png",
        "dateline":"2026-06-24"
    }"#;

    #[test]
    fn parses_iciba_daily_quote() {
        let quote = parse_iciba_response(SAMPLE).expect("valid response");
        assert_eq!(quote.content, "Silence grows in the forest.");
        assert_eq!(quote.note, "寂静在树林里生长。");
        assert_eq!(quote.dateline, "2026-06-24");
        assert_eq!(
            quote.picture_url.as_deref(),
            Some("https://staticedu-wps-cache.iciba.com/image/example.png")
        );
    }

    #[test]
    fn rejects_empty_quote_content() {
        let invalid = r#"{"content":"","note":"空","dateline":"2026-06-24"}"#;
        assert!(parse_iciba_response(invalid).is_err());
    }
}
