use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

const API_URL: &str = "https://open.iciba.com/dsapi/";
const CACHE_FILE: &str = "daily-quote.json";
const IMAGE_BASENAME: &str = "daily-quote";
const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif"];
const MAX_IMAGE_BYTES: u64 = 8 * 1024 * 1024;
const FAILED_REFRESH_RETRY_INTERVAL: Duration = Duration::from_secs(15 * 60);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ImageFormat {
    Jpeg,
    Png,
    WebP,
    Gif,
}

struct PromotedImage {
    path: PathBuf,
    backup_path: Option<PathBuf>,
}

impl ImageFormat {
    fn extension(self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::WebP => "webp",
            Self::Gif => "gif",
        }
    }
}

fn detect_image_format(bytes: &[u8]) -> Option<ImageFormat> {
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        Some(ImageFormat::Jpeg)
    } else if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]) {
        Some(ImageFormat::Png)
    } else if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        Some(ImageFormat::WebP)
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some(ImageFormat::Gif)
    } else {
        None
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DailyQuote {
    pub content: String,
    pub note: String,
    pub dateline: String,
    pub picture_url: Option<String>,
    pub local_image_path: Option<String>,
}

pub fn should_refresh(
    quote_dateline: &str,
    local_date: &str,
    last_attempt_elapsed: Option<Duration>,
) -> bool {
    quote_dateline != local_date
        && last_attempt_elapsed.is_none_or(|elapsed| elapsed >= FAILED_REFRESH_RETRY_INTERVAL)
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

    let mut promoted_image = None;
    if let Some(picture_url) = quote.picture_url.as_deref() {
        match download_image(&agent, picture_url, cache_dir) {
            Ok(image) => {
                quote.local_image_path = Some(image.path.to_string_lossy().into_owned());
                promoted_image = Some(image);
            }
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

    if let Err(error) = write_json_atomic(&cache_dir.join(CACHE_FILE), &quote) {
        if let Some(image) = promoted_image {
            rollback_promoted_image(image);
        }
        return Err(error);
    }
    if let Some(image) = promoted_image {
        finish_promoted_image(cache_dir, image)?;
    }
    Ok(quote)
}

fn download_image(
    agent: &ureq::Agent,
    url: &str,
    cache_dir: &Path,
) -> Result<PromotedImage, Box<dyn Error + Send + Sync>> {
    let response = agent.get(url).call()?;
    let mut bytes = Vec::new();
    response
        .into_reader()
        .take(MAX_IMAGE_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.is_empty() {
        return Err("daily quote image is empty".into());
    }
    if bytes.len() as u64 > MAX_IMAGE_BYTES {
        return Err("daily quote image exceeds 8 MiB".into());
    }

    stage_and_promote_image(cache_dir, &bytes)
}

fn stage_and_promote_image(
    cache_dir: &Path,
    bytes: &[u8],
) -> Result<PromotedImage, Box<dyn Error + Send + Sync>> {
    let format = detect_image_format(bytes).ok_or("unsupported daily quote image format")?;
    let extension = format.extension();
    let staging_path = cache_dir.join(format!("{IMAGE_BASENAME}.new.{extension}"));
    let destination = cache_dir.join(format!("{IMAGE_BASENAME}.{extension}"));
    let backup_path = cache_dir.join(format!("{IMAGE_BASENAME}.previous.{extension}"));

    write_bytes_atomic(&staging_path, bytes)?;
    if slint::Image::load_from_path(&staging_path).is_err() {
        let _ = fs::remove_file(&staging_path);
        return Err("daily quote image cannot be decoded".into());
    }

    let backup = if destination.exists() {
        let _ = fs::remove_file(&backup_path);
        fs::rename(&destination, &backup_path)?;
        Some(backup_path)
    } else {
        None
    };

    if let Err(error) = fs::rename(&staging_path, &destination) {
        if let Some(previous_path) = backup.as_ref() {
            let _ = fs::rename(previous_path, &destination);
        }
        return Err(error.into());
    }

    Ok(PromotedImage {
        path: destination,
        backup_path: backup,
    })
}

fn finish_promoted_image(
    cache_dir: &Path,
    image: PromotedImage,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(backup_path) = image.backup_path {
        fs::remove_file(backup_path)?;
    }
    cleanup_obsolete_images(cache_dir, &image.path)?;
    Ok(())
}

fn rollback_promoted_image(image: PromotedImage) {
    let _ = fs::remove_file(&image.path);
    if let Some(backup_path) = image.backup_path {
        let _ = fs::rename(backup_path, image.path);
    }
}

fn cleanup_obsolete_images(cache_dir: &Path, keep: &Path) -> std::io::Result<()> {
    for extension in IMAGE_EXTENSIONS {
        for candidate_name in [
            format!("{IMAGE_BASENAME}.{extension}"),
            format!("{IMAGE_BASENAME}.new.{extension}"),
            format!("{IMAGE_BASENAME}.previous.{extension}"),
        ] {
            let candidate = cache_dir.join(candidate_name);
            if candidate == keep {
                continue;
            }
            match fs::remove_file(candidate) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
    }

    let interrupted_write = cache_dir.join(format!("{IMAGE_BASENAME}.new.tmp"));
    match fs::remove_file(interrupted_write) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }

    Ok(())
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
    use std::time::{SystemTime, UNIX_EPOCH};

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

    #[test]
    fn current_calendar_day_does_not_refresh_again() {
        assert!(!should_refresh(
            "2026-06-25",
            "2026-06-25",
            Some(Duration::from_secs(24 * 60 * 60))
        ));
    }

    #[test]
    fn stale_quote_refreshes_immediately_on_new_calendar_day() {
        assert!(should_refresh("2026-06-24", "2026-06-25", None));
    }

    #[test]
    fn failed_new_day_refresh_retries_after_fifteen_minutes() {
        assert!(!should_refresh(
            "2026-06-24",
            "2026-06-25",
            Some(Duration::from_secs(14 * 60 + 59))
        ));
        assert!(should_refresh(
            "2026-06-24",
            "2026-06-25",
            Some(Duration::from_secs(15 * 60))
        ));
    }

    #[test]
    fn detects_common_image_formats() {
        let cases: &[(&[u8], &str)] = &[
            (&[0xff, 0xd8, 0xff, 0xe0], "jpg"),
            (&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a], "png"),
            (b"RIFF\x04\x00\x00\x00WEBP", "webp"),
            (b"GIF87a", "gif"),
            (b"GIF89a", "gif"),
        ];

        for (bytes, expected_extension) in cases {
            assert_eq!(
                detect_image_format(bytes).map(ImageFormat::extension),
                Some(*expected_extension)
            );
        }
    }

    #[test]
    fn detects_unsupported_image_data_as_none() {
        assert_eq!(detect_image_format(&[]), None);
        assert_eq!(detect_image_format(b"not an image"), None);
        assert_eq!(detect_image_format(b"RIFFshort"), None);
    }

    #[test]
    fn image_cache_cleanup_preserves_current_image_and_json() {
        let cache_dir = unique_test_cache_dir();
        fs::create_dir_all(&cache_dir).unwrap();
        for name in [
            CACHE_FILE,
            "daily-quote.jpg",
            "daily-quote.jpeg",
            "daily-quote.png",
            "daily-quote.webp",
            "daily-quote.gif",
            "daily-quote.new.png",
            "daily-quote.previous.webp",
        ] {
            fs::write(cache_dir.join(name), b"cached").unwrap();
        }

        let keep = cache_dir.join("daily-quote.jpg");
        cleanup_obsolete_images(&cache_dir, &keep).unwrap();

        assert!(cache_dir.join(CACHE_FILE).exists());
        assert!(keep.exists());
        for name in [
            "daily-quote.jpeg",
            "daily-quote.png",
            "daily-quote.webp",
            "daily-quote.gif",
            "daily-quote.new.png",
            "daily-quote.previous.webp",
        ] {
            assert!(!cache_dir.join(name).exists(), "{name} was not removed");
        }
        fs::remove_dir_all(cache_dir).unwrap();
    }

    #[test]
    fn image_cache_invalid_candidate_preserves_previous_image() {
        let cache_dir = unique_test_cache_dir();
        fs::create_dir_all(&cache_dir).unwrap();
        let previous = cache_dir.join("daily-quote.png");
        fs::write(&previous, b"previous image").unwrap();

        assert!(stage_and_promote_image(&cache_dir, b"not an image").is_err());
        assert_eq!(fs::read(&previous).unwrap(), b"previous image");
        assert_eq!(fs::read_dir(&cache_dir).unwrap().count(), 1);
        fs::remove_dir_all(cache_dir).unwrap();
    }

    fn unique_test_cache_dir() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "raspberry-clock-test-{}-{nonce}",
            std::process::id()
        ))
    }
}
