use background::background_for_date;
use daily_reading::{
    default_cache_dir, fetch_and_cache, load_cache, select_display, should_refresh, DailyReading,
};
use display_power::{apply_screen_power, DisplayPowerState};
use domain::{is_night_screen_window, StartRitual};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

mod background;
mod daily_reading;
mod display_power;
mod domain;

slint::include_modules!();

struct AppState {
    display_power: DisplayPowerState,
    start_ritual: StartRitual,
    reading_sender: Sender<Result<DailyReading, String>>,
    reading_receiver: Receiver<Result<DailyReading, String>>,
    reading_fetch_in_progress: bool,
    last_reading_fetch: Option<Instant>,
    active_reading_date: String,
}

impl AppState {
    fn new(active_reading_date: String) -> Self {
        let (reading_sender, reading_receiver) = mpsc::channel();
        Self {
            display_power: DisplayPowerState::default(),
            start_ritual: StartRitual::default(),
            reading_sender,
            reading_receiver,
            reading_fetch_in_progress: false,
            last_reading_fetch: None,
            active_reading_date,
        }
    }
}

struct ClockSnapshot {
    time_text: String,
    seconds_text: String,
    date_weekday_text: String,
    night_mode: bool,
    night_window: bool,
    timestamp: u64,
    date_key: String,
}

fn main() -> Result<(), slint::PlatformError> {
    std::env::set_var("SLINT_BACKEND", "winit-software");
    std::env::set_var("SLINT_FULLSCREEN", "1");

    let app = AppWindow::new()?;
    let cached_reading = select_display(&load_cache(&default_cache_dir()));
    apply_reading(&app, &cached_reading);
    let state = Rc::new(RefCell::new(AppState::new(active_date_after_apply(
        &cached_reading,
    ))));
    install_touch_wake(&app, state.clone());
    refresh_window(&app, &state);
    start_clock_timer(&app, state);

    app.run()
}

fn install_touch_wake(app: &AppWindow, state: Rc<RefCell<AppState>>) {
    let weak = app.as_weak();
    app.on_screen_tapped(move || {
        let timestamp = unix_timestamp();
        let Some(local_time) = read_local_time(timestamp as libc::time_t) else {
            return;
        };
        let night_window = is_night_screen_window(local_time.tm_hour, local_time.tm_min);
        let (transition, focus_active) = {
            let mut state = state.borrow_mut();
            state.display_power.touch(night_window, timestamp);
            let transition = state.display_power.reconcile(night_window, timestamp);
            state.start_ritual.tap(night_window);
            (transition, state.start_ritual.is_active())
        };
        if let Some(screen_on) = transition {
            apply_screen_power(screen_on);
        }
        if let Some(app) = weak.upgrade() {
            app.set_focus_active(focus_active);
        }
    });
}

fn start_clock_timer(app: &AppWindow, state: Rc<RefCell<AppState>>) {
    let weak = app.as_weak();
    let timer = Box::leak(Box::new(slint::Timer::default()));

    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_secs(1),
        move || {
            if let Some(app) = weak.upgrade() {
                refresh_window(&app, &state);
            }
        },
    );
}

fn refresh_window(app: &AppWindow, state: &Rc<RefCell<AppState>>) {
    apply_reading_updates(app, state);

    let Some(snapshot) = read_clock_snapshot() else {
        return;
    };
    let scene = background_for_date(&snapshot.date_key, snapshot.night_mode);

    app.set_time_text(snapshot.time_text.into());
    app.set_seconds_text(snapshot.seconds_text.into());
    app.set_date_weekday_text(snapshot.date_weekday_text.into());
    app.set_canvas_color(scene.canvas.to_slint_color());
    app.set_wash_primary_color(scene.wash_primary.to_slint_color());
    app.set_wash_secondary_color(scene.wash_secondary.to_slint_color());
    app.set_text_primary_color(scene.text_primary.to_slint_color());
    app.set_text_muted_color(scene.text_muted.to_slint_color());
    app.set_accent_color(scene.accent.to_slint_color());
    app.set_scene_variant(scene.variant.into());
    app.set_night_mode(snapshot.night_mode);

    let (transition, focus_active) = {
        let mut state_ref = state.borrow_mut();
        maybe_start_reading_fetch(&mut state_ref, &snapshot.date_key);
        state_ref.start_ritual.reconcile(snapshot.night_window);
        let focus_active = state_ref.start_ritual.is_active();
        let transition = state_ref
            .display_power
            .reconcile(snapshot.night_window, snapshot.timestamp);
        (transition, focus_active)
    };
    app.set_focus_active(focus_active);
    if let Some(screen_on) = transition {
        apply_screen_power(screen_on);
    }
}

fn read_clock_snapshot() -> Option<ClockSnapshot> {
    let timestamp = unix_timestamp();
    let local_time = read_local_time(timestamp as libc::time_t)?;

    let year = local_time.tm_year + 1900;
    let month = (local_time.tm_mon + 1) as u32;
    let day = local_time.tm_mday as u32;
    let hour = local_time.tm_hour;
    let minute = local_time.tm_min;
    let second = local_time.tm_sec;

    Some(ClockSnapshot {
        time_text: format!("{hour:02}:{minute:02}"),
        seconds_text: format!("{second:02}"),
        date_weekday_text: format!(
            "{year:04}年{month:02}月{day:02}日 · {}",
            weekday_name(local_time.tm_wday)
        ),
        night_mode: !(6..18).contains(&hour),
        night_window: is_night_screen_window(hour, minute),
        timestamp,
        date_key: format!("{year:04}-{month:02}-{day:02}"),
    })
}

fn apply_reading_updates(app: &AppWindow, state: &Rc<RefCell<AppState>>) {
    loop {
        let update = {
            let state_ref = state.borrow();
            state_ref.reading_receiver.try_recv()
        };

        match update {
            Ok(Ok(reading)) => {
                apply_reading(app, &reading);
                let mut state_ref = state.borrow_mut();
                state_ref.active_reading_date = active_date_after_apply(&reading);
                state_ref.reading_fetch_in_progress = false;
            }
            Ok(Err(error)) => {
                eprintln!("daily reading refresh failed: {error}");
                state.borrow_mut().reading_fetch_in_progress = false;
            }
            Err(mpsc::TryRecvError::Empty) => break,
            Err(mpsc::TryRecvError::Disconnected) => {
                state.borrow_mut().reading_fetch_in_progress = false;
                break;
            }
        }
    }
}

fn maybe_start_reading_fetch(state: &mut AppState, date_key: &str) {
    let last_attempt_elapsed = state
        .last_reading_fetch
        .map(|last_fetch| last_fetch.elapsed());
    let refresh_due = should_refresh(&state.active_reading_date, date_key, last_attempt_elapsed);

    if state.reading_fetch_in_progress || !refresh_due {
        return;
    }

    state.reading_fetch_in_progress = true;
    state.last_reading_fetch = Some(Instant::now());
    let sender = state.reading_sender.clone();
    let cache_dir = default_cache_dir();
    let local_date = date_key.to_string();

    std::thread::spawn(move || {
        let result = fetch_and_cache(&cache_dir, &local_date).map_err(|error| error.to_string());
        let _ = sender.send(result);
    });
}

fn apply_reading(app: &AppWindow, reading: &DailyReading) {
    app.set_reading_content(reading.content.clone().into());
    app.set_reading_source(format_reading_source(reading).into());
}

fn format_reading_source(reading: &DailyReading) -> String {
    let origin = truncate_origin(&reading.origin);
    let category = reading
        .category
        .rsplit('-')
        .next()
        .unwrap_or(reading.category.as_str())
        .trim();

    format!("{}《{}》 · {}", reading.author, origin, category)
}

fn truncate_origin(origin: &str) -> String {
    const MAX_SOURCE_CHARS: usize = 20;

    if origin.chars().count() <= MAX_SOURCE_CHARS {
        return origin.to_string();
    }

    origin
        .chars()
        .take(MAX_SOURCE_CHARS - 1)
        .chain(std::iter::once('…'))
        .collect()
}

fn active_date_after_apply(reading: &DailyReading) -> String {
    reading.fetched_for_date.clone()
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn read_local_time(timestamp: libc::time_t) -> Option<libc::tm> {
    let mut local_time = std::mem::MaybeUninit::<libc::tm>::uninit();
    let result = unsafe { libc::localtime_r(&timestamp, local_time.as_mut_ptr()) };

    if result.is_null() {
        None
    } else {
        Some(unsafe { local_time.assume_init() })
    }
}

fn weekday_name(weekday: i32) -> &'static str {
    match weekday {
        0 => "星期日",
        1 => "星期一",
        2 => "星期二",
        3 => "星期三",
        4 => "星期四",
        5 => "星期五",
        _ => "星期六",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_date_after_apply_uses_fetched_for_date() {
        let reading = DailyReading {
            content: "今日诗句".to_string(),
            origin: "白鹿洞二首·其一".to_string(),
            author: "王贞白".to_string(),
            category: "古诗文-人生-读书".to_string(),
            fetched_for_date: "2026-08-30".to_string(),
        };

        assert_eq!(active_date_after_apply(&reading), "2026-08-30");
    }

    #[test]
    fn format_reading_source_includes_author_and_origin() {
        let reading = crate::daily_reading::fallback_reading();
        assert_eq!(
            format_reading_source(&reading),
            "王贞白《白鹿洞二首·其一》 · 读书"
        );
    }

    #[test]
    fn format_reading_source_uses_terminal_category_segment() {
        let reading = DailyReading {
            content: "知之者不如好之者。".to_string(),
            origin: "论语·雍也".to_string(),
            author: "孔子".to_string(),
            category: "古诗文-人生-哲理".to_string(),
            fetched_for_date: "2026-08-30".to_string(),
        };

        assert_eq!(format_reading_source(&reading), "孔子《论语·雍也》 · 哲理");
    }

    #[test]
    fn format_reading_source_truncates_origin_to_twenty_unicode_characters() {
        let reading = DailyReading {
            content: "知之者不如好之者。".to_string(),
            origin: "甲乙丙丁戊己庚辛壬癸子丑寅卯辰巳午未申酉戌".to_string(),
            author: "孔子".to_string(),
            category: "古诗文-人生-哲理".to_string(),
            fetched_for_date: "2026-08-30".to_string(),
        };

        assert_eq!(
            format_reading_source(&reading),
            "孔子《甲乙丙丁戊己庚辛壬癸子丑寅卯辰巳午未申…》 · 哲理"
        );
    }
}
