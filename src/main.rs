use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

slint::include_modules!();

const LONG_PRESS_DELAY: Duration = Duration::from_millis(650);
const READING_MODE_DURATION: Duration = Duration::from_secs(30 * 60);

const MORNING_QUOTES: &[&str] = &[
    "专注当下，今天会慢慢发光。",
    "慢一点，没关系，只要你一直在走。",
    "学而时习之，不亦说乎。",
    "清晨很轻，心也可以很轻。",
    "今天，先把心安顿好。",
];

const AFTERNOON_QUOTES: &[&str] = &[
    "留一点空白，给思绪慢慢舒展。",
    "静下心，读一本书。",
    "此时此刻，便是最好的时光。",
    "让午后的光，照见更平静的自己。",
    "保持热爱，也保持从容。",
];

const EVENING_QUOTES: &[&str] = &[
    "平淡的日子里，也有细碎的光。",
    "慢煮生活，且听风吟。",
    "与其向往远方，不如珍惜身旁。",
    "心如花开，芬芳自来。",
    "夜色温柔，适合把心放缓一点。",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum QuotePeriod {
    Morning,
    Afternoon,
    Evening,
}

struct AppState {
    quote_period: QuotePeriod,
    quote_offset: usize,
    reading_deadline: Option<Instant>,
    pointer_down: bool,
    suppress_tap: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            quote_period: QuotePeriod::Morning,
            quote_offset: 0,
            reading_deadline: None,
            pointer_down: false,
            suppress_tap: false,
        }
    }
}

#[derive(Default)]
struct ClockSnapshot {
    time_text: String,
    seconds_text: String,
    date_text: String,
    weekday_text: String,
    quote_text: String,
    reading_countdown_text: String,
    day_progress: f32,
    year_progress: f32,
    ambient_wave: f32,
    night_mode: bool,
    reading_mode_active: bool,
}

fn main() -> Result<(), slint::PlatformError> {
    std::env::set_var("SLINT_BACKEND", "winit-software");
    std::env::set_var("SLINT_FULLSCREEN", "1");

    let app = AppWindow::new()?;
    let state = Rc::new(RefCell::new(AppState::default()));

    install_interactions(&app, state.clone());
    refresh_window(&app, &state);
    start_clock_timer(&app, state);

    app.run()
}

fn install_interactions(app: &AppWindow, state: Rc<RefCell<AppState>>) {
    let long_press_timer: &'static slint::Timer = Box::leak(Box::new(slint::Timer::default()));

    {
        let weak = app.as_weak();
        let state = state.clone();
        app.on_screen_pressed(move || {
            let mut state_ref = state.borrow_mut();
            state_ref.pointer_down = true;

            if state_ref.reading_deadline.is_some() {
                return;
            }

            drop(state_ref);

            let weak = weak.clone();
            let state = state.clone();
            long_press_timer.stop();
            long_press_timer.start(slint::TimerMode::SingleShot, LONG_PRESS_DELAY, move || {
                let should_enter = {
                    let mut state_ref = state.borrow_mut();
                    if !state_ref.pointer_down || state_ref.reading_deadline.is_some() {
                        false
                    } else {
                        state_ref.reading_deadline = Some(Instant::now() + READING_MODE_DURATION);
                        state_ref.suppress_tap = true;
                        true
                    }
                };

                if should_enter {
                    if let Some(app) = weak.upgrade() {
                        refresh_window(&app, &state);
                    }
                }
            });
        });
    }

    {
        let state = state.clone();
        app.on_screen_released(move || {
            long_press_timer.stop();
            state.borrow_mut().pointer_down = false;
        });
    }

    {
        let weak = app.as_weak();
        app.on_screen_tapped(move || {
            let action = {
                let mut state_ref = state.borrow_mut();
                if state_ref.suppress_tap {
                    state_ref.suppress_tap = false;
                    TapAction::Ignore
                } else if state_ref.reading_deadline.take().is_some() {
                    TapAction::ExitReadingMode
                } else {
                    state_ref.quote_offset = state_ref.quote_offset.wrapping_add(1);
                    TapAction::AdvanceQuote
                }
            };

            if !matches!(action, TapAction::Ignore) {
                if let Some(app) = weak.upgrade() {
                    refresh_window(&app, &state);
                }
            }
        });
    }
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
    let snapshot = {
        let mut state_ref = state.borrow_mut();
        read_clock_snapshot(&mut state_ref)
    };
    apply_snapshot(app, snapshot);
}

fn apply_snapshot(app: &AppWindow, snapshot: ClockSnapshot) {
    app.set_time_text(snapshot.time_text.into());
    app.set_seconds_text(snapshot.seconds_text.into());
    app.set_date_text(snapshot.date_text.into());
    app.set_weekday_text(snapshot.weekday_text.into());
    app.set_quote_text(snapshot.quote_text.into());
    app.set_reading_countdown_text(snapshot.reading_countdown_text.into());
    app.set_reading_caption_text("静心阅读".into());
    app.set_day_progress(snapshot.day_progress);
    app.set_year_progress(snapshot.year_progress);
    app.set_ambient_wave(snapshot.ambient_wave);
    app.set_night_mode(snapshot.night_mode);
    app.set_reading_mode_active(snapshot.reading_mode_active);
}

fn read_clock_snapshot(state: &mut AppState) -> ClockSnapshot {
    let now = SystemTime::now();
    let timestamp = match now.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs(),
        Err(_) => 0,
    };

    let local_time = match read_local_time(timestamp as libc::time_t) {
        Some(local_time) => local_time,
        None => return ClockSnapshot::default(),
    };

    sync_state_with_time(state, &local_time);

    let hour = local_time.tm_hour;
    let minute = local_time.tm_min;
    let second = local_time.tm_sec;

    let seconds_from_midnight = (hour * 3600 + minute * 60 + second) as f32;
    let day_progress = (seconds_from_midnight / 86_400.0).clamp(0.0, 1.0);
    let days_in_year = if is_leap_year(local_time.tm_year + 1900) {
        366.0
    } else {
        365.0
    };
    let year_progress = ((local_time.tm_yday as f32 + day_progress) / days_in_year).clamp(0.0, 1.0);

    let quote = pick_quote(state, &local_time).to_string();
    let ambient_wave = triangle_wave(timestamp, 24);

    let reading_countdown = reading_countdown_text(state.reading_deadline);

    ClockSnapshot {
        time_text: format!("{:02}:{:02}", hour, minute),
        seconds_text: format!("{:02}", second),
        date_text: format!(
            "{:04}年{:02}月{:02}日",
            local_time.tm_year + 1900,
            local_time.tm_mon + 1,
            local_time.tm_mday
        ),
        weekday_text: weekday_name(local_time.tm_wday).to_string(),
        quote_text: quote,
        reading_countdown_text: reading_countdown,
        day_progress,
        year_progress,
        ambient_wave,
        night_mode: is_night_mode(hour),
        reading_mode_active: state.reading_deadline.is_some(),
    }
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

fn sync_state_with_time(state: &mut AppState, local_time: &libc::tm) {
    let current_period = quote_period_for_hour(local_time.tm_hour);
    if state.quote_period != current_period {
        state.quote_period = current_period;
        state.quote_offset = 0;
    }

    if let Some(deadline) = state.reading_deadline {
        if Instant::now() >= deadline {
            state.reading_deadline = None;
        }
    }
}

fn pick_quote<'a>(state: &AppState, local_time: &libc::tm) -> &'a str {
    let quotes = quote_list(state.quote_period);
    let seed =
        ((local_time.tm_year + 1900) as usize * 367) + local_time.tm_yday as usize + period_index(state.quote_period);
    let index = (seed + state.quote_offset) % quotes.len();
    quotes[index]
}

fn quote_list(period: QuotePeriod) -> &'static [&'static str] {
    match period {
        QuotePeriod::Morning => MORNING_QUOTES,
        QuotePeriod::Afternoon => AFTERNOON_QUOTES,
        QuotePeriod::Evening => EVENING_QUOTES,
    }
}

fn period_index(period: QuotePeriod) -> usize {
    match period {
        QuotePeriod::Morning => 0,
        QuotePeriod::Afternoon => 1,
        QuotePeriod::Evening => 2,
    }
}

fn quote_period_for_hour(hour: i32) -> QuotePeriod {
    match hour {
        5..=11 => QuotePeriod::Morning,
        12..=17 => QuotePeriod::Afternoon,
        _ => QuotePeriod::Evening,
    }
}

fn is_night_mode(hour: i32) -> bool {
    !(6..18).contains(&hour)
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

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn triangle_wave(timestamp: u64, period_seconds: u64) -> f32 {
    if period_seconds == 0 {
        return 0.0;
    }

    let phase = (timestamp % period_seconds) as f32 / period_seconds as f32;
    if phase < 0.5 {
        phase * 2.0
    } else {
        (1.0 - phase) * 2.0
    }
}

fn reading_countdown_text(deadline: Option<Instant>) -> String {
    let Some(deadline) = deadline else {
        return "30:00".to_string();
    };

    let remaining = deadline.saturating_duration_since(Instant::now());
    let total_seconds = remaining.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;

    format!("{:02}:{:02}", minutes, seconds)
}

enum TapAction {
    Ignore,
    AdvanceQuote,
    ExitReadingMode,
}
