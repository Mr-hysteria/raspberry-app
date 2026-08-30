use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

const CACHE_FILE: &str = "daily-reading.json";
const CACHE_VERSION: u8 = 1;
const FAILED_REFRESH_RETRY_INTERVAL: Duration = Duration::from_secs(15 * 60);
const MAX_RESPONSE_BODY_BYTES: usize = 64 * 1024;
const MAX_CONTENT_CHARS: usize = 40;
// Metadata budgets are Unicode-character limits. They preserve long work/category names while
// bounding both persisted cache slots independently of UTF-8 byte width.
const MAX_ORIGIN_CHARS: usize = 80;
const MAX_AUTHOR_CHARS: usize = 40;
const MAX_CATEGORY_CHARS: usize = 80;
const ENDPOINTS: [&str; 5] = [
    "https://v1.jinrishici.com/rensheng/dushu.json",
    "https://v1.jinrishici.com/rensheng/zheli.json",
    "https://v1.jinrishici.com/shanshui.json",
    "https://v1.jinrishici.com/shenghuo/tianyuan.json",
    "https://v1.jinrishici.com/rensheng/lizhi.json",
];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DailyReading {
    pub content: String,
    pub origin: String,
    pub author: String,
    pub category: String,
    pub fetched_for_date: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ReadingCache {
    pub version: u8,
    pub current: Option<DailyReading>,
    pub previous: Option<DailyReading>,
}

#[derive(Deserialize)]
struct JinrishiciResponse {
    #[serde(default)]
    content: String,
    #[serde(default)]
    origin: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    category: String,
}

pub fn default_cache_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cache")
        .join("raspberry-clock")
}

pub fn fallback_reading() -> DailyReading {
    DailyReading {
        content: "读书不觉已春深，一寸光阴一寸金。".to_string(),
        origin: "白鹿洞二首·其一".to_string(),
        author: "王贞白".to_string(),
        category: "古诗文-人生-读书".to_string(),
        fetched_for_date: String::new(),
    }
}

pub fn load_cache(cache_dir: &Path) -> ReadingCache {
    let content = match fs::read_to_string(cache_dir.join(CACHE_FILE)) {
        Ok(content) => content,
        Err(_) => return ReadingCache::default(),
    };
    let cache: ReadingCache = match serde_json::from_str(&content) {
        Ok(cache) => cache,
        Err(_) => return ReadingCache::default(),
    };
    if cache.version == CACHE_VERSION {
        cache
    } else {
        ReadingCache::default()
    }
}

pub fn select_display(cache: &ReadingCache) -> DailyReading {
    cache
        .current
        .clone()
        .or_else(|| cache.previous.clone())
        .unwrap_or_else(fallback_reading)
}

pub fn should_refresh(
    reading_date: &str,
    local_date: &str,
    last_attempt_elapsed: Option<Duration>,
) -> bool {
    reading_date != local_date
        && last_attempt_elapsed.is_none_or(|elapsed| elapsed >= FAILED_REFRESH_RETRY_INTERVAL)
}

pub fn fetch_and_cache(
    cache_dir: &Path,
    local_date: &str,
) -> Result<DailyReading, Box<dyn Error + Send + Sync>> {
    fs::create_dir_all(cache_dir)?;
    let cache = load_cache(cache_dir);
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(8))
        .timeout_read(Duration::from_secs(12))
        .build();
    let response = agent.get(&endpoint_for_date(local_date)).call()?;
    let json = read_response_body(response.into_reader())?;
    let reading = parse_response(&json, local_date)?;
    let updated = update_cache(cache, reading.clone());
    persist_reading_and_cleanup(cache_dir, &updated, reading)
}

fn read_response_body(reader: impl Read) -> Result<String, Box<dyn Error + Send + Sync>> {
    let mut body = Vec::with_capacity(MAX_RESPONSE_BODY_BYTES + 1);
    reader
        .take((MAX_RESPONSE_BODY_BYTES + 1) as u64)
        .read_to_end(&mut body)?;
    if body.len() > MAX_RESPONSE_BODY_BYTES {
        return Err(
            format!("daily reading response exceeds {MAX_RESPONSE_BODY_BYTES} bytes").into(),
        );
    }
    Ok(String::from_utf8(body)?)
}

fn persist_reading_and_cleanup(
    cache_dir: &Path,
    cache: &ReadingCache,
    reading: DailyReading,
) -> Result<DailyReading, Box<dyn Error + Send + Sync>> {
    write_cache_atomic(cache_dir, cache)?;
    if let Err(error) = cleanup_legacy_cache(cache_dir) {
        eprintln!("daily reading cache committed, but legacy cache cleanup failed: {error}");
    }
    Ok(reading)
}

fn parse_response(
    json: &str,
    local_date: &str,
) -> Result<DailyReading, Box<dyn Error + Send + Sync>> {
    let response: JinrishiciResponse = serde_json::from_str(json)?;
    let reading = DailyReading {
        content: required_field("content", response.content)?,
        origin: required_metadata("origin", response.origin, MAX_ORIGIN_CHARS)?,
        author: required_metadata("author", response.author, MAX_AUTHOR_CHARS)?,
        category: required_metadata("category", response.category, MAX_CATEGORY_CHARS)?,
        fetched_for_date: local_date.to_string(),
    };

    if !content_is_suitable(&reading.content) {
        return Err("daily reading content is not suitable".into());
    }

    Ok(reading)
}

fn endpoint_for_date(local_date: &str) -> String {
    let index = (fnv1a_hash(local_date.as_bytes()) % ENDPOINTS.len() as u64) as usize;
    ENDPOINTS[index].to_string()
}

fn content_is_suitable(content: &str) -> bool {
    let trimmed = content.trim();
    !trimmed.is_empty()
        && trimmed.chars().count() <= MAX_CONTENT_CHARS
        && !["惆怅", "愁", "恨", "泪", "悲", "死", "亡", "病", "战", "悔"]
            .iter()
            .any(|needle| trimmed.contains(needle))
}

fn required_field(
    field_name: &'static str,
    value: String,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(format!("daily reading response missing {field_name}").into())
    } else {
        Ok(trimmed.to_string())
    }
}

fn required_metadata(
    field_name: &'static str,
    value: String,
    max_chars: usize,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let value = required_field(field_name, value)?;
    if value.chars().count() > max_chars {
        Err(format!("daily reading {field_name} exceeds {max_chars} Unicode characters").into())
    } else {
        Ok(value)
    }
}

fn fnv1a_hash(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn update_cache(mut cache: ReadingCache, reading: DailyReading) -> ReadingCache {
    cache.version = CACHE_VERSION;
    match cache.current.take() {
        Some(current) if current.fetched_for_date != reading.fetched_for_date => {
            cache.previous = Some(current);
            cache.current = Some(reading);
        }
        Some(_) => {
            cache.current = Some(reading);
        }
        None => {
            cache.current = Some(reading);
        }
    }
    cache
}

fn write_cache_atomic(
    cache_dir: &Path,
    cache: &ReadingCache,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let destination = cache_dir.join(CACHE_FILE);
    let temporary = cache_dir.join(format!("{CACHE_FILE}.tmp"));
    let json = serde_json::to_vec(cache)?;
    let mut file = fs::File::create(&temporary)?;
    file.write_all(&json)?;
    file.sync_all()?;
    fs::rename(temporary, destination)?;
    Ok(())
}

fn cleanup_legacy_cache(cache_dir: &Path) -> std::io::Result<()> {
    let mut candidates = vec![
        "daily-quote.json".to_string(),
        "daily-quote.new.tmp".to_string(),
    ];
    for extension in ["jpg", "jpeg", "png", "webp", "gif"] {
        candidates.push(format!("daily-quote.{extension}"));
        candidates.push(format!("daily-quote.new.{extension}"));
        candidates.push(format!("daily-quote.previous.{extension}"));
    }

    let mut failures = Vec::new();
    for candidate in candidates {
        let path = cache_dir.join(candidate);
        match fs::remove_file(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => failures.push(format!("{}: {error}", path.display())),
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            failures.join("; "),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Cursor;
    use std::path::PathBuf;
    use std::time::Duration;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parses_complete_jinrishici_response() {
        let json = r#"{
            "content":"幼敏悟过人，读书辄成诵。",
            "origin":"宋史·列传",
            "author":"脱脱",
            "category":"古诗文-人生-读书"
        }"#;

        let reading = parse_response(json, "2026-08-30").expect("valid reading");

        assert_eq!(
            reading,
            DailyReading {
                content: "幼敏悟过人，读书辄成诵。".to_string(),
                origin: "宋史·列传".to_string(),
                author: "脱脱".to_string(),
                category: "古诗文-人生-读书".to_string(),
                fetched_for_date: "2026-08-30".to_string(),
            }
        );
    }

    #[test]
    fn rejects_each_missing_required_field() {
        let cases = [
            (
                "content",
                r#"{"content":" ","origin":"宋史·列传","author":"脱脱","category":"古诗文-人生-读书"}"#,
            ),
            (
                "origin",
                r#"{"content":"幼敏悟过人，读书辄成诵。","origin":" ","author":"脱脱","category":"古诗文-人生-读书"}"#,
            ),
            (
                "author",
                r#"{"content":"幼敏悟过人，读书辄成诵。","origin":"宋史·列传","author":" ","category":"古诗文-人生-读书"}"#,
            ),
            (
                "category",
                r#"{"content":"幼敏悟过人，读书辄成诵。","origin":"宋史·列传","author":"脱脱","category":""}"#,
            ),
        ];

        for (field, json) in cases {
            assert!(
                parse_response(json, "2026-08-30").is_err(),
                "{field} should be required"
            );
        }
    }

    #[test]
    fn rejects_content_longer_than_forty_unicode_characters() {
        let long_content = "春".repeat(41);
        let json = format!(
            r#"{{"content":"{long_content}","origin":"长歌行","author":"佚名","category":"古诗文-人生-读书"}}"#
        );

        assert!(parse_response(&json, "2026-08-30").is_err());
    }

    #[test]
    fn rejects_low_arousal_unsafe_content() {
        for banned in ["惆怅", "愁", "恨", "泪", "悲", "死", "亡", "病", "战", "悔"] {
            let json = format!(
                r#"{{"content":"山中{banned}不可闻。","origin":"杂诗","author":"某氏","category":"古诗文-人生-哲理"}}"#
            );

            assert!(
                parse_response(&json, "2026-08-30").is_err(),
                "{banned} should be rejected"
            );
            assert!(!content_is_suitable(&format!("山中{banned}不可闻。")));
        }
    }

    #[test]
    fn accepts_calm_reading_content() {
        assert!(content_is_suitable("幼敏悟过人，读书辄成诵。"));
    }

    #[test]
    fn rejects_response_when_the_sixty_five_thousand_five_hundred_thirty_seventh_byte_exists() {
        let body = vec![b'x'; 65_537];

        let error = read_response_body(Cursor::new(body)).expect_err("body must be rejected");

        assert_eq!(
            error.to_string(),
            "daily reading response exceeds 65536 bytes"
        );
    }

    #[test]
    fn rejects_each_oversized_metadata_field_by_unicode_character_count() {
        let cases = [
            (
                "origin",
                format!(
                    r#"{{"content":"幼敏悟过人，读书辄成诵。","origin":"{}","author":"欧阳修","category":"古诗文-人生-读书"}}"#,
                    "卷".repeat(81)
                ),
                "daily reading origin exceeds 80 Unicode characters",
            ),
            (
                "author",
                format!(
                    r#"{{"content":"幼敏悟过人，读书辄成诵。","origin":"画地学书","author":"{}","category":"古诗文-人生-读书"}}"#,
                    "欧".repeat(41)
                ),
                "daily reading author exceeds 40 Unicode characters",
            ),
            (
                "category",
                format!(
                    r#"{{"content":"幼敏悟过人，读书辄成诵。","origin":"画地学书","author":"欧阳修","category":"{}"}}"#,
                    "类".repeat(81)
                ),
                "daily reading category exceeds 80 Unicode characters",
            ),
        ];

        for (field, json, expected_error) in cases {
            let error = parse_response(&json, "2026-08-30")
                .expect_err("oversized metadata must be rejected");
            assert_eq!(error.to_string(), expected_error, "wrong error for {field}");
        }
    }

    #[test]
    fn date_route_is_deterministic_and_whitelisted() {
        let whitelist = [
            "https://v1.jinrishici.com/rensheng/dushu.json",
            "https://v1.jinrishici.com/rensheng/zheli.json",
            "https://v1.jinrishici.com/shanshui.json",
            "https://v1.jinrishici.com/shenghuo/tianyuan.json",
            "https://v1.jinrishici.com/rensheng/lizhi.json",
        ];

        for date in [
            "2026-08-30",
            "2026-08-31",
            "2026-09-01",
            "2027-01-01",
            "2030-12-31",
        ] {
            let first = endpoint_for_date(date);
            let second = endpoint_for_date(date);

            assert_eq!(first, second, "{date} should be stable");
            assert!(
                whitelist.contains(&first.as_str()),
                "{date} produced unexpected endpoint {first}"
            );
        }
    }

    #[test]
    fn current_calendar_day_does_not_refresh_again() {
        assert!(!should_refresh(
            "2026-08-30",
            "2026-08-30",
            Some(Duration::from_secs(24 * 60 * 60))
        ));
    }

    #[test]
    fn new_calendar_day_refreshes_immediately() {
        assert!(should_refresh("2026-08-29", "2026-08-30", None));
    }

    #[test]
    fn failed_refresh_retries_after_fifteen_minutes() {
        assert!(!should_refresh(
            "2026-08-29",
            "2026-08-30",
            Some(Duration::from_secs(14 * 60 + 59))
        ));
        assert!(should_refresh(
            "2026-08-29",
            "2026-08-30",
            Some(Duration::from_secs(15 * 60))
        ));
    }

    #[test]
    fn first_success_populates_current_only() {
        let updated = update_cache(
            ReadingCache::default(),
            reading("2026-08-30", "读书辄成诵。"),
        );

        assert_eq!(updated.version, 1);
        assert_eq!(updated.current, Some(reading("2026-08-30", "读书辄成诵。")));
        assert_eq!(updated.previous, None);
    }

    #[test]
    fn new_date_moves_current_to_previous() {
        let cache = ReadingCache {
            version: 1,
            current: Some(reading("2026-08-30", "读书辄成诵。")),
            previous: None,
        };

        let updated = update_cache(cache, reading("2026-08-31", "一寸光阴一寸金。"));

        assert_eq!(
            updated.current,
            Some(reading("2026-08-31", "一寸光阴一寸金。"))
        );
        assert_eq!(
            updated.previous,
            Some(reading("2026-08-30", "读书辄成诵。"))
        );
    }

    #[test]
    fn same_date_replaces_current_without_rotating_previous() {
        let cache = ReadingCache {
            version: 1,
            current: Some(reading("2026-08-30", "旧内容")),
            previous: Some(reading("2026-08-29", "更旧内容")),
        };

        let updated = update_cache(cache, reading("2026-08-30", "新内容"));

        assert_eq!(updated.current, Some(reading("2026-08-30", "新内容")));
        assert_eq!(updated.previous, Some(reading("2026-08-29", "更旧内容")));
    }

    #[test]
    fn select_display_prefers_current_then_previous_then_fallback() {
        let current = reading("2026-08-30", "当前内容");
        let previous = reading("2026-08-29", "上一条内容");

        assert_eq!(
            select_display(&ReadingCache {
                version: 1,
                current: Some(current.clone()),
                previous: Some(previous.clone()),
            }),
            current
        );
        assert_eq!(
            select_display(&ReadingCache {
                version: 1,
                current: None,
                previous: Some(previous.clone()),
            }),
            previous
        );
        assert_eq!(select_display(&ReadingCache::default()), fallback_reading());
    }

    #[test]
    fn invalid_cache_file_returns_empty_cache() {
        let cache_dir = unique_test_cache_dir();
        fs::create_dir_all(&cache_dir).unwrap();
        fs::write(cache_dir.join("daily-reading.json"), b"{not json").unwrap();

        assert_eq!(load_cache(&cache_dir), ReadingCache::default());

        fs::remove_dir_all(cache_dir).unwrap();
    }

    #[test]
    fn atomic_write_round_trips_two_slot_cache() {
        let cache_dir = unique_test_cache_dir();
        fs::create_dir_all(&cache_dir).unwrap();
        let cache = ReadingCache {
            version: 1,
            current: Some(reading("2026-08-30", "当前内容")),
            previous: Some(reading("2026-08-29", "上一条内容")),
        };

        write_cache_atomic(&cache_dir, &cache).unwrap();

        assert_eq!(load_cache(&cache_dir), cache);

        fs::remove_dir_all(cache_dir).unwrap();
    }

    #[test]
    fn legacy_cleanup_removes_only_known_quote_files() {
        let cache_dir = unique_test_cache_dir();
        fs::create_dir_all(&cache_dir).unwrap();
        for name in [
            "daily-quote.json",
            "daily-quote.jpg",
            "daily-quote.jpeg",
            "daily-quote.png",
            "daily-quote.webp",
            "daily-quote.gif",
            "daily-quote.new.jpg",
            "daily-quote.new.png",
            "daily-quote.previous.webp",
            "daily-quote.previous.gif",
            "daily-quote.new.tmp",
            "unrelated.txt",
            "daily-reading.json",
        ] {
            fs::write(cache_dir.join(name), b"x").unwrap();
        }

        cleanup_legacy_cache(&cache_dir).unwrap();

        for removed in [
            "daily-quote.json",
            "daily-quote.jpg",
            "daily-quote.jpeg",
            "daily-quote.png",
            "daily-quote.webp",
            "daily-quote.gif",
            "daily-quote.new.jpg",
            "daily-quote.new.png",
            "daily-quote.previous.webp",
            "daily-quote.previous.gif",
            "daily-quote.new.tmp",
        ] {
            assert!(
                !cache_dir.join(removed).exists(),
                "{removed} should be removed"
            );
        }
        for kept in ["unrelated.txt", "daily-reading.json"] {
            assert!(cache_dir.join(kept).exists(), "{kept} should remain");
        }

        fs::remove_dir_all(cache_dir).unwrap();
    }

    #[test]
    fn committed_reading_succeeds_when_known_legacy_candidate_is_undeletable() {
        let cache_dir = unique_test_cache_dir();
        fs::create_dir_all(cache_dir.join("daily-quote.json")).unwrap();
        fs::write(cache_dir.join("daily-quote.png"), b"legacy").unwrap();
        fs::write(cache_dir.join("unrelated.txt"), b"keep me").unwrap();
        let new_reading = reading("2026-08-31", "一寸光阴一寸金。");
        let updated = update_cache(ReadingCache::default(), new_reading.clone());

        let result = persist_reading_and_cleanup(&cache_dir, &updated, new_reading.clone());

        assert_eq!(
            result.expect("committed reading stays successful"),
            new_reading
        );
        assert_eq!(load_cache(&cache_dir), updated);
        assert_eq!(
            fs::read(cache_dir.join("unrelated.txt")).unwrap(),
            b"keep me"
        );
        assert!(!cache_dir.join("daily-quote.png").exists());

        fs::remove_dir_all(cache_dir).unwrap();
    }

    fn reading(fetched_for_date: &str, content: &str) -> DailyReading {
        DailyReading {
            content: content.to_string(),
            origin: "白鹿洞二首·其一".to_string(),
            author: "王贞白".to_string(),
            category: "古诗文-人生-读书".to_string(),
            fetched_for_date: fetched_for_date.to_string(),
        }
    }

    fn unique_test_cache_dir() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "raspberry-clock-reading-test-{}-{nonce}",
            std::process::id()
        ))
    }
}
