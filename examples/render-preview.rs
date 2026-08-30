#[path = "../src/background.rs"]
mod background;

use background::background_for_date;
use std::cell::RefCell;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

slint::include_modules!();

const WIDTH: u32 = 800;
const HEIGHT: u32 = 480;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("SLINT_BACKEND", "winit-software");
    std::env::set_var("SLINT_SCALE_FACTOR", "1");

    let (state, output_path) = parse_args()?;

    let app = AppWindow::new()?;
    let scene = scene_for_state(&state)?;
    let snapshot_status = Rc::new(RefCell::new(None));

    app.window().set_size(slint::PhysicalSize::new(WIDTH, HEIGHT));
    app.set_time_text("09:41".into());
    app.set_seconds_text("27".into());
    app.set_date_weekday_text("2026年08月30日 · 星期日".into());
    app.set_reading_content("幼敏悟过人，读书辄成诵。".into());
    app.set_reading_source("欧阳修《画地学书》 · 读书".into());
    app.set_focus_active(state == "focus");
    app.set_canvas_color(scene.canvas.to_slint_color());
    app.set_wash_primary_color(scene.wash_primary.to_slint_color());
    app.set_wash_secondary_color(scene.wash_secondary.to_slint_color());
    app.set_text_primary_color(scene.text_primary.to_slint_color());
    app.set_text_muted_color(scene.text_muted.to_slint_color());
    app.set_accent_color(scene.accent.to_slint_color());
    app.set_scene_variant(scene.variant.into());
    app.set_night_mode(state == "night");
    app.set_ui_font_family(preview_font_family().into());

    schedule_snapshot(app.as_weak(), output_path, snapshot_status.clone());
    app.show()?;
    slint::run_event_loop_until_quit()?;

    let outcome = snapshot_status.borrow_mut().take();
    match outcome {
        Some(Ok(())) => Ok(()),
        Some(Err(error)) => Err(error.into()),
        None => Err("snapshot timer did not fire".into()),
    }
}

fn parse_args() -> Result<(String, PathBuf), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let state = args.next().ok_or_else(usage_error)?;
    let output_path = args.next().map(PathBuf::from).ok_or_else(usage_error)?;

    if args.next().is_some() {
        return Err(usage_error().into());
    }

    Ok((state, output_path))
}

fn scene_for_state(state: &str) -> Result<background::BackgroundScene, Box<dyn std::error::Error>> {
    match state {
        "day" | "focus" => Ok(background_for_date("2026-08-30", false)),
        "night" => Ok(background_for_date("2026-08-30", true)),
        _ => Err(format!("unknown preview state: {state}").into()),
    }
}

fn schedule_snapshot(
    app: slint::Weak<AppWindow>,
    output_path: PathBuf,
    snapshot_status: Rc<RefCell<Option<Result<(), String>>>>,
) {
    let timer = Box::leak(Box::new(slint::Timer::default()));

    timer.start(
        slint::TimerMode::SingleShot,
        Duration::from_millis(150),
        move || {
            let result = capture_snapshot(&app, &output_path);
            *snapshot_status.borrow_mut() = Some(result);
            let _ = slint::quit_event_loop();
        },
    );
}

fn capture_snapshot(app: &slint::Weak<AppWindow>, output_path: &Path) -> Result<(), String> {
    let app = app
        .upgrade()
        .ok_or_else(|| "preview window was dropped before snapshot".to_string())?;
    let snapshot = app
        .window()
        .take_snapshot()
        .map_err(|error| format!("failed to take snapshot: {error}"))?;

    if snapshot.width() != WIDTH || snapshot.height() != HEIGHT {
        return Err(format!(
            "snapshot size mismatch: expected {WIDTH}x{HEIGHT}, got {}x{}",
            snapshot.width(),
            snapshot.height()
        ));
    }

    write_ppm(&snapshot, output_path).map_err(|error| format!("failed to write PPM: {error}"))
}

fn write_ppm(
    snapshot: &slint::SharedPixelBuffer<slint::Rgba8Pixel>,
    output_path: &Path,
) -> std::io::Result<()> {
    let mut file = std::fs::File::create(output_path)?;
    write!(file, "P6\n{} {}\n255\n", snapshot.width(), snapshot.height())?;

    for pixel in snapshot.as_slice() {
        file.write_all(&[pixel.r, pixel.g, pixel.b])?;
    }

    file.flush()
}

fn usage_error() -> String {
    "usage: render-preview <day|focus|night> <output.ppm>".to_string()
}

fn preview_font_family() -> String {
    if let Ok(font_family) = std::env::var("RASPBERRY_PREVIEW_FONT") {
        if !font_family.trim().is_empty() {
            return font_family;
        }
    }

    #[cfg(target_os = "macos")]
    {
        "Hiragino Sans GB".to_string()
    }

    #[cfg(not(target_os = "macos"))]
    {
        "WenQuanYi Zen Hei".to_string()
    }
}
