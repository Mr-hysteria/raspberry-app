# Daily Quote Image Cache Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reliably display JPEG, PNG, WebP, and GIF daily-quote images while retaining at most one valid cached background.

**Architecture:** Keep image format detection and cache cleanup in `src/daily_quote.rs`. Download into a format-specific staging file, validate it with Slint, atomically promote it, persist the matching JSON path, and only then remove obsolete known variants; failures preserve the prior cache.

**Tech Stack:** Rust, Slint 1.13.1, ureq 2.12, standard filesystem APIs

---

### Task 1: Detect supported image formats

**Files:**
- Modify: `src/daily_quote.rs`

- [ ] **Step 1: Write failing signature-detection tests**

Add tests asserting JPEG (`FF D8 FF`), PNG, WebP (`RIFF....WEBP`), and GIF87a/GIF89a map to `jpg`, `png`, `webp`, and `gif`, while empty and arbitrary bytes are rejected.

- [ ] **Step 2: Verify the tests fail**

Run: `cargo test daily_quote::tests::detects_ -- --nocapture`
Expected: compilation failure because `detect_image_format` does not exist.

- [ ] **Step 3: Implement minimal signature detection**

Add a private `ImageFormat` enum with `extension()` and a `detect_image_format(bytes: &[u8]) -> Option<ImageFormat>` function using exact magic-byte checks.

- [ ] **Step 4: Verify detection tests pass**

Run: `cargo test daily_quote::tests::detects_ -- --nocapture`
Expected: all format-detection tests pass.

### Task 2: Promote validated images and remove obsolete variants

**Files:**
- Modify: `src/daily_quote.rs`

- [ ] **Step 1: Write failing cache-lifecycle tests**

Use a unique temporary directory to assert that cleanup preserves `daily-quote.json` and the selected image while deleting other known image variants. Add a test asserting unsupported candidate bytes leave the previous image untouched.

- [ ] **Step 2: Verify lifecycle tests fail**

Run: `cargo test daily_quote::tests::image_cache_ -- --nocapture`
Expected: compilation failure because the cache lifecycle helpers do not exist.

- [ ] **Step 3: Implement staged validation and cleanup**

Download at most 8 MiB, reject a response that reaches the limit without EOF, detect its format, write `daily-quote.new.<extension>`, validate with `slint::Image::load_from_path`, then atomically rename it to `daily-quote.<extension>`. Implement cleanup over only `jpg`, `jpeg`, `png`, `webp`, and `gif` variants, excluding the selected path.

- [ ] **Step 4: Integrate cache persistence and rollback**

Make `fetch_and_cache` preserve the prior image until candidate validation succeeds. Persist JSON with the promoted path, remove obsolete variants only after JSON succeeds, and restore the prior same-extension image if JSON persistence fails.

- [ ] **Step 5: Verify lifecycle tests pass**

Run: `cargo test daily_quote::tests::image_cache_ -- --nocapture`
Expected: all lifecycle tests pass.

### Task 3: Verify the application

**Files:**
- Modify: `docs/iteration-log.md`

- [ ] **Step 1: Record the compatibility fix**

Document the JPEG-as-PNG root cause, supported formats, single-image cache policy, and failure fallback.

- [ ] **Step 2: Format and run all automated checks**

Run: `cargo fmt --check && cargo test && for test_script in tests/*.sh; do bash "$test_script"; done && cargo build --release`
Expected: formatting is clean, all Rust and shell tests pass, and the release build exits successfully.

- [ ] **Step 3: Inspect the final diff**

Run: `git diff --check && git status --short && git diff --stat`
Expected: no whitespace errors and only planned files are modified.
