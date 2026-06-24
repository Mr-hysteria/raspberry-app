# Focus Clock Interface Design

## Goal

Replace the current rhythm-tree interface with a concise 800×480 focus clock that emphasizes time, annual progress, the CPA exam countdown, and a bilingual daily quote.

## Visual hierarchy

- Use the ICIBA daily quote PNG as a full-screen background.
- Apply a dark left-to-right gradient overlay so text stays readable across varying images.
- Place the large hour and minute display at the upper left, with seconds visually reduced.
- Place the full date and weekday below the time.
- Place two translucent cards at the upper right:
  - percentage of the current year remaining;
  - days until the next August 29 CPA exam date.
- Place the English quote and Chinese note at the lower left.
- Remove the daily rhythm tree, decorative plant, reading mode, long-press behavior, and local quote switching.
- Preserve a subdued day/night color adjustment without adding decorative elements.

## Daily quote data

- Request `https://open.iciba.com/dsapi/` when the application starts.
- Refresh after the local date changes or after 24 hours, whichever requires a newer quote.
- Perform network and image downloads on a worker thread so the one-second clock refresh remains responsive.
- Cache one JSON record and one downloaded background image locally.
- On request or image failure, retain the last successful cached quote.
- If no cache exists, show a built-in bilingual quote over a plain gradient background.

## Countdown calculations

- Display the percentage of the current calendar year still remaining.
- Calculate the CPA countdown to the nearest upcoming August 29 using local calendar dates.
- On or after the exam date has elapsed, automatically target August 29 of the following year.
- On August 29 itself, display zero days remaining.

## Night screen behavior

- From 23:30 through 06:59 local time, the display is normally off.
- At 07:00 it returns to the normal always-on state.
- Touching the screen during the night interval wakes it for 60 seconds.
- Any further touch during that minute restarts the 60-second timer.
- Screen power is controlled through X11 DPMS commands, with failures logged rather than terminating the clock.

## Architecture

- `src/domain.rs`: pure date, percentage, countdown, and night-window calculations.
- `src/daily_quote.rs`: ICIBA response parsing, cache paths, download, and fallback selection.
- `src/display_power.rs`: DPMS state transitions and temporary wake deadline.
- `src/main.rs`: Slint lifecycle, one-second snapshots, background worker communication, and touch callbacks.
- `ui/clock.slint`: presentation only.

## Error handling

- A malformed API response does not replace valid cached data.
- A failed image download does not remove the previously cached background.
- Cache write failures are non-fatal and leave in-memory content visible.
- DPMS command failures are non-fatal.

## Verification

- Unit tests cover leap-year percentage, CPA countdown before/on/after August 29, the 23:30–07:00 interval, temporary wake expiry, and ICIBA JSON parsing.
- `cargo test` and `cargo build --release` must pass locally.
- The cross-compiled binary must deploy and start on the Raspberry Pi.
- Remote logs must be checked after launch.
