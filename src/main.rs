use daily_quote::{
    default_cache_dir, fallback_quote, fetch_and_cache, load_cached, should_refresh, DailyQuote,
};
use display_power::{apply_screen_power, DisplayPowerState};
use domain::{days_until_cpa, is_night_screen_window, year_remaining_fraction};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

mod daily_quote;
mod display_power;
mod domain;

slint::include_modules!();

struct AppState {
    display_power: DisplayPowerState,
    quote_sender: Sender<Result<DailyQuote, String>>,
    quote_receiver: Receiver<Result<DailyQuote, String>>,
    quote_fetch_in_progress: bool,
    last_quote_fetch: Option<Instant>,
    active_quote_date: String,
}

impl AppState {
    fn new(active_quote_date: String) -> Self {
        let (quote_sender, quote_receiver) = mpsc::channel();
        Self {
            display_power: DisplayPowerState::default(),
            quote_sender,
            quote_receiver,
            quote_fetch_in_progress: false,
            last_quote_fetch: None,
            active_quote_date,
        }
    }
}

struct ClockSnapshot {
    time_text: String,
    seconds_text: String,
    date_text: String,
    weekday_text: String,
    year_remaining_text: String,
    year_remaining_progress: f32,
    cpa_countdown_text: String,
    night_mode: bool,
    night_window: bool,
    timestamp: u64,
    date_key: String,
}

fn main() -> Result<(), slint::PlatformError> {
    std::env::set_var("SLINT_BACKEND", "winit-software");
    std::env::set_var("SLINT_FULLSCREEN", "1");

    let app = AppWindow::new()?;
    let cached_quote = load_cached(&default_cache_dir()).unwrap_or_else(fallback_quote);
    let state = Rc::new(RefCell::new(AppState::new(cached_quote.dateline.clone())));
    apply_quote(&app, &cached_quote);
    install_touch_wake(&app, state.clone());
    refresh_window(&app, &state);
    start_clock_timer(&app, state);

    app.run()
}

fn install_touch_wake(app: &AppWindow, state: Rc<RefCell<AppState>>) {
    app.on_screen_tapped(move || {
        let timestamp = unix_timestamp();
        let Some(local_time) = read_local_time(timestamp as libc::time_t) else {
            return;
        };
        let night_window = is_night_screen_window(local_time.tm_hour, local_time.tm_min);
        let transition = {
            let mut state = state.borrow_mut();
            state.display_power.touch(night_window, timestamp);
            state.display_power.reconcile(night_window, timestamp)
        };
        if let Some(screen_on) = transition {
            apply_screen_power(screen_on);
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
    apply_quote_updates(app, state);

    let Some(snapshot) = read_clock_snapshot() else {
        return;
    };

    app.set_time_text(snapshot.time_text.into());
    app.set_seconds_text(snapshot.seconds_text.into());
    app.set_date_text(snapshot.date_text.into());
    app.set_weekday_text(snapshot.weekday_text.into());
    app.set_year_remaining_text(snapshot.year_remaining_text.into());
    app.set_year_remaining_progress(snapshot.year_remaining_progress);
    app.set_cpa_countdown_text(snapshot.cpa_countdown_text.into());
    app.set_cpa_date_text("考试日期 · 8月29日".into());
    app.set_night_mode(snapshot.night_mode);

    let transition = {
        let mut state_ref = state.borrow_mut();
        maybe_start_quote_fetch(&mut state_ref, &snapshot.date_key);
        state_ref
            .display_power
            .reconcile(snapshot.night_window, snapshot.timestamp)
    };
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
    let seconds_today = (hour * 3600 + minute * 60 + second) as u32;
    let year_remaining =
        year_remaining_fraction(year, local_time.tm_yday as u32 + 1, seconds_today);
    let cpa_days = days_until_cpa(year, month, day);

    Some(ClockSnapshot {
        time_text: format!("{hour:02}:{minute:02}"),
        seconds_text: format!("{second:02}"),
        date_text: format!("{year:04}年{month:02}月{day:02}日"),
        weekday_text: weekday_name(local_time.tm_wday).to_string(),
        year_remaining_text: format!("今年还剩 {:.0}%", year_remaining * 100.0),
        year_remaining_progress: year_remaining,
        cpa_countdown_text: format!("还有 {cpa_days} 天"),
        night_mode: !(6..18).contains(&hour),
        night_window: is_night_screen_window(hour, minute),
        timestamp,
        date_key: format!("{year:04}-{month:02}-{day:02}"),
    })
}

fn apply_quote_updates(app: &AppWindow, state: &Rc<RefCell<AppState>>) {
    loop {
        let update = {
            let state_ref = state.borrow();
            state_ref.quote_receiver.try_recv()
        };

        match update {
            Ok(Ok(quote)) => {
                apply_quote(app, &quote);
                let mut state_ref = state.borrow_mut();
                state_ref.active_quote_date = quote.dateline;
                state_ref.quote_fetch_in_progress = false;
            }
            Ok(Err(error)) => {
                eprintln!("daily quote refresh failed: {error}");
                state.borrow_mut().quote_fetch_in_progress = false;
            }
            Err(mpsc::TryRecvError::Empty) => break,
            Err(mpsc::TryRecvError::Disconnected) => {
                state.borrow_mut().quote_fetch_in_progress = false;
                break;
            }
        }
    }
}

fn maybe_start_quote_fetch(state: &mut AppState, date_key: &str) {
    let last_attempt_elapsed = state
        .last_quote_fetch
        .map(|last_fetch| last_fetch.elapsed());
    let refresh_due = should_refresh(&state.active_quote_date, date_key, last_attempt_elapsed);

    if state.quote_fetch_in_progress || !refresh_due {
        return;
    }

    state.quote_fetch_in_progress = true;
    state.last_quote_fetch = Some(Instant::now());
    let sender = state.quote_sender.clone();
    let cache_dir = default_cache_dir();

    std::thread::spawn(move || {
        let result = fetch_and_cache(&cache_dir).map_err(|error| error.to_string());
        let _ = sender.send(result);
    });
}

fn apply_quote(app: &AppWindow, quote: &DailyQuote) {
    app.set_quote_english(quote.content.clone().into());
    app.set_quote_chinese(quote.note.clone().into());

    let image = quote
        .local_image_path
        .as_deref()
        .filter(|path| Path::new(path).exists())
        .and_then(|path| slint::Image::load_from_path(Path::new(path)).ok());

    app.set_has_background_image(image.is_some());
    app.set_background_image(image.unwrap_or_default());
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
