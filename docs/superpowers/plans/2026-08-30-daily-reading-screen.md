# 每日阅读屏实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 CPA 倒计时海报改造成由今日诗词、本地程序化背景和一触开始仪式组成的低干扰桌面阅读屏，并在 Raspberry Pi Zero 2 W 真机完成验证。

**Architecture:** 保留现有每秒 Slint 刷新、后台线程和 X11 DPMS 链路；用 `daily_reading` 模块替换 ICIBA 图片链路，用两个 JSON 槽位提供离线回退，用纯色值的 `background` 模块驱动 Slint 程序化水墨背景。开始仪式只保存在运行期 `StartRitual` 状态，不计时、不落盘；网络、内容、背景和交互逻辑都在 UI 外形成可单测边界。

**Tech Stack:** Rust 2021、Slint 1.13.1、ureq 2.12、serde/serde_json、X11 DPMS、Bash 验证脚本

**Spec:** `docs/superpowers/specs/2026-08-30-daily-reading-screen-design.md`

## Global Constraints

- 目标设备固定为 Raspberry Pi Zero 2 W、512MB 内存、Debian 13 64-bit、800×480 X11 屏幕。
- 运行时不得请求、解码或缓存任何远程图片；AIC 不进入代码路径。
- 今日诗词请求必须在后台线程中执行，每天成功最多一次，失败重试间隔不少于 15 分钟。
- 内容只能来自规格中列出的五个今日诗词 v1 分类端点，并必须通过必填字段、40 字长度和负向词过滤。
- 磁盘只保留 `current` 与 `previous` 两条阅读内容，不保存历史列表。
- 23:30–07:00 自动息屏和夜间触摸唤醒 60 秒的现有行为必须保持。
- 白天轻触只切换无计时、无记录的开始状态；进入夜间窗口时清除该状态。
- UI 统一使用目标设备已经安装的 `WenQuanYi Zen Hei`，不新增字体包或字体资源。
- 不新增运行时 crate、数据库、异步运行时、动画或常驻网络连接。
- Slint 声明式布局用编译、`take_snapshot()` 实际渲染和人工截图检查验证，不新增只匹配源码字符串的 UI 测试。

---

### Task 1: Replace ICIBA with a filtered daily-reading source and two-slot cache

**Files:**
- Create: `src/daily_reading.rs`
- Delete: `src/daily_quote.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Produces: `DailyReading { content, origin, author, category, fetched_for_date }` and `ReadingCache { version, current, previous }`.
- Produces: `default_cache_dir() -> PathBuf`, `fallback_reading() -> DailyReading`, `load_cache(&Path) -> ReadingCache`, `select_display(&ReadingCache) -> DailyReading`, `should_refresh(&str, &str, Option<Duration>) -> bool`, and `fetch_and_cache(&Path, &str) -> Result<DailyReading, Box<dyn Error + Send + Sync>>`.
- Produces: private pure helpers `parse_response(&str, &str)`, `endpoint_for_date(&str)`, `content_is_suitable(&str)`, `update_cache(ReadingCache, DailyReading)`, `write_cache_atomic`, and `cleanup_legacy_cache` exercised by module tests.
- Consumes later: Task 3 imports the six public interfaces from `daily_reading` and sends `DailyReading` through the existing background-thread channel.

- [ ] **Step 1: Write failing parser, routing, filter and refresh tests**

  In `src/daily_reading.rs`, first add tests with literal fixtures for these behaviors:

  - `parses_complete_jinrishici_response`: JSON containing `content`, `origin`, `author`, and `category` becomes a `DailyReading` stamped with the supplied `2026-08-30` local date.
  - `rejects_each_missing_required_field`: four table rows independently blank each required field and must return an error.
  - `rejects_content_longer_than_forty_unicode_characters`: a 41-character Chinese literal must return an error.
  - `rejects_low_arousal_unsafe_content`: literals containing `惆怅`, `愁`, `恨`, `泪`, `悲`, `死`, `亡`, `病`, `战`, and `悔` must each be rejected.
  - `accepts_calm_reading_content`: `幼敏悟过人，读书辄成诵。` must be accepted.
  - `date_route_is_deterministic_and_whitelisted`: repeated calls for one date are equal and selected URLs for five fixed dates all belong to the exact five-URL whitelist.
  - `current_calendar_day_does_not_refresh_again`, `new_calendar_day_refreshes_immediately`, and `failed_refresh_retries_after_fifteen_minutes` preserve the existing retry contract.

- [ ] **Step 2: Run the focused tests and verify RED**

  Run: `cargo test daily_reading::tests -- --nocapture`

  Expected: compilation fails because the new module functions and types are not implemented.

- [ ] **Step 3: Implement response parsing, content filtering, deterministic routing and fallback**

  Implement these exact data shapes:

  ```rust
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
  ```

  Use `CACHE_VERSION = 1`, `MAX_CONTENT_CHARS = 40`, the five exact spec URLs, and `FAILED_REFRESH_RETRY_INTERVAL = 15 minutes`. Compute the route from a stable byte-wise FNV-1a hash of the `YYYY-MM-DD` string; do not use Rust's randomized `HashMap` hasher. The built-in fallback is `读书不觉已春深，一寸光阴一寸金。` / `白鹿洞二首·其一` / `王贞白` / `古诗文-人生-读书` with an empty date.

- [ ] **Step 4: Run focused tests and verify GREEN**

  Run: `cargo test daily_reading::tests -- --nocapture`

  Expected: parser, route, filter and refresh tests pass.

- [ ] **Step 5: Write failing two-slot cache and cleanup tests**

  Add tests using a unique directory under `std::env::temp_dir()`:

  - `first_success_populates_current_only`.
  - `new_date_moves_current_to_previous`.
  - `same_date_replaces_current_without_rotating_previous`.
  - `select_display_prefers_current_then_previous_then_fallback`.
  - `invalid_cache_file_returns_empty_cache`.
  - `atomic_write_round_trips_two_slot_cache`.
  - `legacy_cleanup_removes_only_known_quote_files`: remove `daily-quote.json`, `daily-quote.jpg/jpeg/png/webp/gif`, `.new.*`, `.previous.*`, and `.new.tmp`, while preserving `unrelated.txt` and `daily-reading.json`.

- [ ] **Step 6: Run cache tests and verify RED**

  Run: `cargo test daily_reading::tests -- --nocapture`

  Expected: the new cache tests fail because cache rotation, persistence and cleanup are absent.

- [ ] **Step 7: Implement cache lifecycle and one-request network refresh**

  Implement `daily-reading.json` with a same-directory `.tmp` file, `sync_all()`, and `fs::rename`. `fetch_and_cache(cache_dir, local_date)` must create the directory, issue exactly one `ureq` GET to `endpoint_for_date(local_date)` with 8-second connect and 12-second read timeouts, parse and filter the response, rotate a cloned cache, atomically persist it, then clean known legacy files. A failed request, parse, filter, or write must leave the last valid cache untouched.

- [ ] **Step 8: Run focused and full tests**

  Run: `cargo test daily_reading::tests -- --nocapture && cargo test`

  Expected: all new tests and the full suite pass after old ICIBA/image tests are removed.

- [ ] **Step 9: Commit Task 1**

  ```bash
  git add src/daily_reading.rs src/daily_quote.rs src/main.rs
  git commit -m "feat: add filtered daily reading cache"
  ```

---

### Task 2: Add deterministic background palettes and the start ritual state

**Files:**
- Create: `src/background.rs`
- Modify: `src/domain.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Produces: `RgbaColor { r, g, b, a }`, `BackgroundScene { canvas, wash_primary, wash_secondary, text_primary, text_muted, accent, variant }`, and `background_for_date(&str, bool) -> BackgroundScene`.
- Produces: `StartRitual::is_active()`, `StartRitual::tap(night_window)`, and `StartRitual::reconcile(night_window)`.
- Consumes later: Task 3 converts `RgbaColor` through `to_slint_color()` and binds the resulting scene plus `StartRitual` state to `AppWindow` properties.

- [ ] **Step 1: Write failing background tests**

  Add literal tests proving:

  - `same_date_and_mode_produce_same_scene`.
  - `fixed_dates_cover_more_than_one_day_palette` using `2026-08-30`, `2026-08-31`, `2026-09-01`, and `2026-09-02`.
  - `all_scene_colors_are_opaque_or_intentionally_translucent`: canvas/text alpha is 255; wash alpha is between 20 and 96.
  - `night_canvas_is_darker_than_day_canvas` using integer luminance `299*r + 587*g + 114*b`.
  - `variant_is_always_zero_through_three`.

- [ ] **Step 2: Run background tests and verify RED**

  Run: `cargo test background::tests -- --nocapture`

  Expected: compilation fails because `background` types/functions do not exist.

- [ ] **Step 3: Implement four day palettes and four night palettes**

  Implement `RgbaColor::to_slint_color()` with `slint::Color::from_argb_u8(a, r, g, b)`. Select palette and `variant` from the same stable FNV-1a date hash modulo four. Use only static byte color constants; no filesystem, clock, network, random generator or animation state.

- [ ] **Step 4: Run background tests and verify GREEN**

  Run: `cargo test background::tests -- --nocapture`

  Expected: all background tests pass.

- [ ] **Step 5: Write failing start-ritual tests**

  Add these tests to `src/domain.rs`:

  - `daytime_tap_enters_and_second_tap_exits_start_ritual`.
  - `nighttime_tap_does_not_change_start_ritual`.
  - `entering_night_window_clears_start_ritual`.
  - `daytime_reconcile_preserves_start_ritual`.

- [ ] **Step 6: Run ritual tests and verify RED**

  Run: `cargo test domain::tests -- --nocapture`

  Expected: compilation fails because `StartRitual` does not exist.

- [ ] **Step 7: Implement minimal runtime-only start state**

  Add `#[derive(Default)] pub struct StartRitual { active: bool }`. `tap(true)` is a no-op; `tap(false)` toggles. `reconcile(true)` clears; `reconcile(false)` preserves. No timestamp, counter, duration, persistence or analytics field is allowed.

- [ ] **Step 8: Run focused and full tests**

  Run: `cargo test background::tests -- --nocapture && cargo test domain::tests -- --nocapture && cargo test`

  Expected: all tests pass.

- [ ] **Step 9: Commit Task 2**

  ```bash
  git add src/background.rs src/domain.rs src/main.rs
  git commit -m "feat: add calm background and start ritual state"
  ```

---

### Task 3: Integrate the reading model into the 800×480 Slint screen

**Files:**
- Modify: `src/main.rs`
- Modify: `ui/clock.slint`
- Delete: `tests/ui-font-sizes.sh`

**Interfaces:**
- Consumes: Task 1 `DailyReading`/cache/fetch interfaces and Task 2 `background_for_date`/`StartRitual`.
- Produces in Slint: `reading-content`, `reading-source`, `focus-active`, six scene colors, and `scene-variant` properties plus the existing `time-text`, `seconds-text`, `date-weekday-text`, `night-mode`, and `screen-tapped` callback.

- [ ] **Step 1: Record the declarative-UI verification ruling**

  In the SDD ledger, record: `Ruling: Slint layout is declarative UI; verify it by compiler success and actual 800×480 snapshots instead of a source-grep test — a layout regression missed by snapshot inspection would require rework.` This follows the plan's global constraint and replaces the obsolete source-matching `tests/ui-font-sizes.sh`.

- [ ] **Step 2: Rewire `main.rs` to the new modules and run the compiler to verify RED**

  Replace `mod daily_quote` with `mod daily_reading` and add `mod background`. Change the channel to `Sender/Receiver<Result<DailyReading, String>>`, load/select the two-slot cache at startup, and remove image fields and `refresh_date_after_apply`. Before changing the Slint file, set the planned reading/background properties from Rust.

  Run: `cargo check`

  Expected: compilation fails because the old Slint component lacks the new reading and scene property setters.

- [ ] **Step 3: Implement the complete reading-screen layout**

  Replace the old photo/card hierarchy with a fixed `800×480` composition:

  - Full-screen canvas using `canvas-color` plus three low-alpha rounded wash shapes whose positions are selected by `scene-variant`.
  - Date at `(64, 38)` with at least 20px font.
  - Time at approximately `(58, 68)`, at least 108px font, with seconds at most 22px.
  - “今日一页” label and short accent line near `y=238`.
  - Reading content in a `650×104` area beginning near `(64, 270)`, 29–32px font, word wrap, at most three lines under the 40-character contract.
  - Source at `y≈382`, at least 19px font; Rust formats it as `作者《作品》 · 分类末级` and truncates the work name to 20 Unicode characters with `…`.
  - Bottom prompt at `y≈430`; normal text is `轻触一下，先读一页`, active text is `已经开始了，先读一页就好`.
  - All text uses `WenQuanYi Zen Hei`; no `Noto Serif CJK SC`, image property, CPA property, year property, card or progress bar remains.

  In focus state, keep time fully visible and reduce reading/source contrast by selecting `text-muted-color`; do not hide the content or add animation.

- [ ] **Step 4: Integrate tap and nightly reconciliation**

  Add `start_ritual` to `AppState`. In `screen-tapped`, keep the existing display-power touch/reconcile call, then toggle `StartRitual` only when `night_window == false` and immediately update `focus-active`. In each clock refresh, call `start_ritual.reconcile(snapshot.night_window)` so the state clears at night. Do not change `DisplayPowerState`, its 60-second deadline or `is_night_screen_window`.

- [ ] **Step 5: Compile and run all logic tests**

  Run: `cargo fmt --check && cargo test && cargo check`

  Expected: formatting, all tests and Slint compilation pass.

- [ ] **Step 6: Remove obsolete source-grep UI test and run shell tests**

  Delete `tests/ui-font-sizes.sh`; the remaining configuration scripts must still pass:

  Run: `for test_script in tests/*.sh; do bash "$test_script"; done`

  Expected: autostart, run-clock and watchdog configuration tests pass.

- [ ] **Step 7: Commit Task 3**

  ```bash
  git add src/main.rs ui/clock.slint tests/ui-font-sizes.sh
  git commit -m "feat: redesign clock as daily reading screen"
  ```

---

### Task 4: Add an actual Slint snapshot harness and inspect three states

**Files:**
- Create: `examples/render-preview.rs`
- Create: `scripts/render-previews.sh`

**Interfaces:**
- Consumes: Task 3 Slint properties and Task 2 background scene.
- Produces: command `scripts/render-previews.sh OUTPUT_DIR` and files `reading-day.png`, `reading-focus.png`, `reading-night.png`, each exactly 800×480.

- [ ] **Step 1: Write the preview example against the real component and verify RED**

  Add `examples/render-preview.rs` with `slint::include_modules!()`, include `../src/background.rs` by path, and accept exactly two arguments: state (`day`, `focus`, or `night`) and output PPM path. Populate fixed fixture values `09:41`, `27`, `2026年08月30日 · 星期日`, `幼敏悟过人，读书辄成诵。`, and `欧阳修《画地学书》 · 读书`. Set the real scene properties and window size to `PhysicalSize::new(800, 480)`.

  Run: `cargo check --example render-preview`

  Expected: compilation fails until snapshot scheduling and PPM writing are implemented.

- [ ] **Step 2: Implement snapshot capture with standard-library PPM output**

  Show the component, start a one-shot Slint timer after 150ms, call `window().take_snapshot()`, assert buffer width/height are 800/480, write a binary P6 PPM header plus RGB bytes derived from the RGBA buffer, then call `slint::quit_event_loop()`. Return non-zero with a clear error for an unknown state, wrong argument count, snapshot failure or wrong size. Do not add an image crate.

- [ ] **Step 3: Add the host conversion script**

  `scripts/render-previews.sh OUTPUT_DIR` must use `mktemp -d`, run the example once per state, and convert PPM to PNG using `sips` on macOS or ImageMagick `convert` on Linux. It must fail if neither converter exists, validate the final dimensions using `sips -g pixelWidth -g pixelHeight` or `identify`, and remove only its own temporary directory through a trap.

- [ ] **Step 4: Build and generate all screenshots**

  Run: `cargo check --example render-preview && scripts/render-previews.sh /tmp/raspberry-clock-previews`

  Expected: three PNG files are produced at exactly 800×480.

- [ ] **Step 5: Inspect screenshots**

  Open all three PNGs at original detail. Verify every visual acceptance criterion from the spec: no clipping, time hierarchy, readable source, normal/active prompt change, focus contrast reduction, night palette, no CPA/year/photo remnants. If any criterion fails, adjust `ui/clock.slint`, regenerate all three and repeat inspection.

- [ ] **Step 6: Commit Task 4**

  ```bash
  git add examples/render-preview.rs scripts/render-previews.sh ui/clock.slint
  git commit -m "test: add reading screen visual previews"
  ```

---

### Task 5: Update product, architecture, operation and iteration documentation

**Files:**
- Modify: `README.md`
- Modify: `agent.md`
- Modify: `开发SOP.md`
- Modify: `docs/prd-healing-life.md`
- Modify: `docs/decisions.md`
- Modify: `docs/iteration-log.md`
- Create: `docs/architecture.md`

**Interfaces:**
- Consumes: final file names, cache schema, API endpoints, UI behavior and verification commands from Tasks 1–4.
- Produces: one consistent maintenance description; no document may continue claiming CPA, annual progress, ICIBA or remote background pictures are current features.

- [ ] **Step 1: Update user-facing and agent entry documents**

  In `README.md` and `agent.md`, describe the daily-reading screen, five whitelisted API routes, two-slot cache, programmatic background, start ritual and unchanged night behavior. Update the project structure to list `daily_reading.rs`, `background.rs`, `domain.rs`, `display_power.rs`, the preview example and preview script.

- [ ] **Step 2: Update the PRD to v3**

  Replace the current CPA/ICIBA scope in `docs/prd-healing-life.md` with the approved v3 product definition and acceptance criteria from the spec. Preserve the low-distraction, no manual input and Raspberry Pi constraints.

- [ ] **Step 3: Record architecture and decision rationale**

  Create `docs/architecture.md` with the clock tick, background-thread fetch/channel, JSON cache selection, Slint property binding, touch/start ritual and DPMS state flows. Add a decision entry in `docs/decisions.md` explaining why today-poetry plus programmatic background replaced ICIBA/AIC, including the real Pi AIC 403 evidence and tradeoffs.

- [ ] **Step 4: Update operations and iteration log**

  In `开发SOP.md`, document `daily-reading.json`, safe cache inspection/removal, preview generation and the final verification commands without adding credentials or fixed IPs. In `docs/iteration-log.md`, record the redesign, tests, screenshots, cross-build result and measured real-device evidence available at implementation time.

- [ ] **Step 5: Check documentation consistency**

  Run: `rg -n "CPA|金山词霸|ICIBA|背景图|daily-quote" README.md agent.md docs/prd-healing-life.md docs/architecture.md 开发SOP.md`

  Expected: matches only occur in explicitly labeled history/migration/troubleshooting context, never in the current-feature description.

- [ ] **Step 6: Commit Task 5**

  ```bash
  git add README.md agent.md 开发SOP.md docs/prd-healing-life.md docs/decisions.md docs/iteration-log.md docs/architecture.md
  git commit -m "docs: document daily reading screen"
  ```

---

## Final verification and deployment

- [x] Run `cargo fmt --check`.
- [x] Run `cargo test` and record the exact passed-test count.
- [x] Run every `tests/*.sh` script and record the exact passed-script count.
- [x] Run `cargo build --release`.
- [x] Run `cargo build --release --target aarch64-unknown-linux-gnu`.
- [x] Run `cargo check --example render-preview` and regenerate/inspect all three 800×480 PNGs.
- [x] Run `git diff --check`, `git status --short`, and inspect the complete branch diff against its merge base.
- [x] Dispatch a whole-branch code review; fix every Critical/Important issue and re-review the fix range once.
- [x] Deploy through `scripts/deploy-and-run-pi.sh` or an equivalent explicit binary upload/restart sequence.
- [x] On the Pi, verify the running executable, `800×480` screenshot, online `daily-reading.json`, absence of legacy image cache, RSS, process uptime, no-proxy API response, white-space/clipping, start ritual touch behavior and X11 DPMS command path.
- [x] Re-run the relevant automated commands after any review or true-device fix.
- [x] Update `docs/iteration-log.md` with final measured evidence and commit it.
