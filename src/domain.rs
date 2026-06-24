use chrono::NaiveDate;

pub fn year_remaining_fraction(year: i32, ordinal: u32, seconds_today: u32) -> f32 {
    let days = if is_leap_year(year) { 366_u64 } else { 365_u64 };
    let total_seconds = days * 86_400;
    let elapsed_seconds =
        (ordinal.saturating_sub(1) as u64 * 86_400 + seconds_today as u64).min(total_seconds);
    (total_seconds - elapsed_seconds) as f32 / total_seconds as f32
}

pub fn days_until_cpa(year: i32, month: u32, day: u32) -> i64 {
    let today = NaiveDate::from_ymd_opt(year, month, day).expect("valid local calendar date");
    let this_year_exam = NaiveDate::from_ymd_opt(year, 8, 29).expect("valid CPA date");
    let target = if today <= this_year_exam {
        this_year_exam
    } else {
        NaiveDate::from_ymd_opt(year + 1, 8, 29).expect("valid next CPA date")
    };

    (target - today).num_days()
}

pub fn is_night_screen_window(hour: i32, minute: i32) -> bool {
    hour > 23 || (hour == 23 && minute >= 30) || hour < 7
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn year_remaining_reaches_zero_at_end_of_last_day() {
        let remaining = year_remaining_fraction(2026, 365, 86_400);
        assert_eq!(remaining, 0.0);
    }

    #[test]
    fn leap_year_remaining_uses_366_days() {
        let remaining = year_remaining_fraction(2028, 60, 0);
        let expected = 307.0 / 366.0;
        assert!((remaining - expected).abs() < 0.0001);
    }

    #[test]
    fn cpa_countdown_is_zero_on_exam_day() {
        assert_eq!(days_until_cpa(2026, 8, 29), 0);
    }

    #[test]
    fn cpa_countdown_rolls_to_next_year_after_exam() {
        assert_eq!(days_until_cpa(2026, 8, 30), 364);
    }

    #[test]
    fn night_window_starts_at_2330_and_ends_at_0700() {
        assert!(!is_night_screen_window(23, 29));
        assert!(is_night_screen_window(23, 30));
        assert!(is_night_screen_window(6, 59));
        assert!(!is_night_screen_window(7, 0));
    }
}
