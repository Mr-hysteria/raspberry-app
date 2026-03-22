use std::time::Duration;

slint::include_modules!();

#[derive(Default)]
struct ClockSnapshot {
    time_text: String,
    seconds_text: String,
    date_text: String,
    weekday_text: String,
}

fn main() -> Result<(), slint::PlatformError> {
    std::env::set_var("SLINT_BACKEND", "winit-software");
    std::env::set_var("SLINT_FULLSCREEN", "1");

    let app = AppWindow::new()?;
    apply_snapshot(&app, &read_clock_snapshot());
    start_clock_timer(&app);
    app.run()
}

fn start_clock_timer(app: &AppWindow) {
    let weak = app.as_weak();
    let timer = Box::leak(Box::new(slint::Timer::default()));

    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_secs(1),
        move || {
            if let Some(app) = weak.upgrade() {
                apply_snapshot(&app, &read_clock_snapshot());
            }
        },
    );
}

fn apply_snapshot(app: &AppWindow, snapshot: &ClockSnapshot) {
    app.set_time_text(snapshot.time_text.clone().into());
    app.set_seconds_text(snapshot.seconds_text.clone().into());
    app.set_date_text(snapshot.date_text.clone().into());
    app.set_weekday_text(snapshot.weekday_text.clone().into());
}

fn read_clock_snapshot() -> ClockSnapshot {
    let timestamp = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => duration.as_secs() as libc::time_t,
        Err(_) => 0,
    };

    let mut local_time = std::mem::MaybeUninit::<libc::tm>::uninit();

    let result = unsafe { libc::localtime_r(&timestamp, local_time.as_mut_ptr()) };
    if result.is_null() {
        return ClockSnapshot::default();
    }

    let local_time = unsafe { local_time.assume_init() };
    let weekday_text = match local_time.tm_wday {
        0 => "星期日",
        1 => "星期一",
        2 => "星期二",
        3 => "星期三",
        4 => "星期四",
        5 => "星期五",
        _ => "星期六",
    };

    ClockSnapshot {
        time_text: format!("{:02}:{:02}", local_time.tm_hour, local_time.tm_min),
        seconds_text: format!("{:02}", local_time.tm_sec),
        date_text: format!(
            "{:04}年{:02}月{:02}日",
            local_time.tm_year + 1900,
            local_time.tm_mon + 1,
            local_time.tm_mday
        ),
        weekday_text: weekday_text.to_string(),
    }
}
