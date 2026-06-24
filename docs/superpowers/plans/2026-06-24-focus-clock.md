# Focus Clock Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver and deploy the approved immersive-poster clock with ICIBA daily quotes, year remaining, CPA countdown, and scheduled night screen power.

**Architecture:** Keep calendar and screen-schedule logic pure and unit tested. Fetch/cache the daily quote on a worker thread and send updates to the Slint event loop. Keep the Slint file limited to the approved 800×480 presentation.

**Tech Stack:** Rust 2021, Slint 1.13.1, serde/serde_json, ureq with rustls, X11 `xset`, Cargo tests.

---

### Task 1: Pure calendar and night-window behavior

**Files:**
- Create: `src/domain.rs`
- Modify: `src/main.rs`

- [ ] Write tests for year remaining, CPA days before/on/after August 29, and the 23:30–07:00 night interval.
- [ ] Run `cargo test domain -- --nocapture` and verify the new tests fail because the functions do not exist.
- [ ] Implement `year_remaining_percent`, `days_until_cpa`, and `is_night_screen_window` using local calendar components.
- [ ] Run `cargo test domain -- --nocapture` and verify all domain tests pass.

### Task 2: Daily quote parsing and persistent cache

**Files:**
- Create: `src/daily_quote.rs`
- Modify: `Cargo.toml`
- Modify: `src/main.rs`

- [ ] Write tests that parse the supplied ICIBA response and reject responses missing quote content.
- [ ] Run `cargo test daily_quote -- --nocapture` and verify failure before implementation.
- [ ] Add serde, serde_json, and a blocking HTTPS client with rustls.
- [ ] Implement startup cache loading, atomic JSON/image replacement, API download, and built-in fallback data.
- [ ] Run `cargo test daily_quote -- --nocapture` and verify the parser/cache tests pass.

### Task 3: Night display power state

**Files:**
- Create: `src/display_power.rs`
- Modify: `src/main.rs`

- [ ] Write tests for a 60-second wake deadline, touch extension, expiry, and daytime reset.
- [ ] Run `cargo test display_power -- --nocapture` and verify failure before implementation.
- [ ] Implement state transitions that issue `xset dpms force off` and `xset dpms force on` only when the desired power state changes.
- [ ] Run `cargo test display_power -- --nocapture` and verify all state tests pass.

### Task 4: Approved immersive-poster Slint UI

**Files:**
- Replace: `ui/clock.slint`
- Modify: `src/main.rs`

- [ ] Add Slint properties for bilingual quote, year remaining label/progress, CPA countdown/date, and optional background image.
- [ ] Remove daily rhythm, plant, reading mode, long-press, and quote-advance interaction.
- [ ] Implement the approved A layout: time/date upper left, metric cards upper right, bilingual quote lower left, full-screen image plus readability overlays.
- [ ] Wire worker-thread quote updates through `slint::invoke_from_event_loop`.
- [ ] Run `cargo test` and `cargo build` and resolve all compile/test failures.

### Task 5: Documentation and deployment

**Files:**
- Modify: `README.md`
- Modify: `agent.md`
- Modify: `docs/prd-healing-life.md`
- Modify: `docs/iteration-log.md`

- [ ] Update product and agent documentation to match the removed and added behavior.
- [ ] Run `cargo fmt --check`, `cargo test`, and `cargo build --release`.
- [ ] Run `./scripts/deploy-and-run-pi.sh`.
- [ ] Verify the remote process and inspect `/tmp/raspberry-clock.log`.
- [ ] Review the final diff for accidental files, secrets, and unrelated changes.

## Self-review

- Every approved visual, data, countdown, fallback, and night-screen requirement maps to a task.
- No database, configuration UI, or unrelated refactor is included.
- Network work remains outside the UI timer.
