pub fn is_night_screen_window(hour: i32, minute: i32) -> bool {
    hour > 23 || (hour == 23 && minute >= 30) || hour < 7
}

#[derive(Default)]
pub struct StartRitual {
    active: bool,
}

impl StartRitual {
    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn tap(&mut self, night_window: bool) {
        if night_window {
            return;
        }

        self.active = !self.active;
    }

    pub fn reconcile(&mut self, night_window: bool) {
        if night_window {
            self.active = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn night_window_starts_at_2330_and_ends_at_0700() {
        assert!(!is_night_screen_window(23, 29));
        assert!(is_night_screen_window(23, 30));
        assert!(is_night_screen_window(6, 59));
        assert!(!is_night_screen_window(7, 0));
    }

    #[test]
    fn daytime_tap_enters_and_second_tap_exits_start_ritual() {
        let mut ritual = StartRitual::default();

        ritual.tap(false);
        assert!(ritual.is_active());

        ritual.tap(false);
        assert!(!ritual.is_active());
    }

    #[test]
    fn nighttime_tap_does_not_change_start_ritual() {
        let mut ritual = StartRitual::default();

        ritual.tap(true);

        assert!(!ritual.is_active());
    }

    #[test]
    fn entering_night_window_clears_start_ritual() {
        let mut ritual = StartRitual::default();
        ritual.tap(false);

        ritual.reconcile(true);

        assert!(!ritual.is_active());
    }

    #[test]
    fn daytime_reconcile_preserves_start_ritual() {
        let mut ritual = StartRitual::default();
        ritual.tap(false);

        ritual.reconcile(false);

        assert!(ritual.is_active());
    }
}
