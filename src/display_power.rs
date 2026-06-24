use std::process::Command;

const NIGHT_WAKE_SECONDS: u64 = 60;

pub struct DisplayPowerState {
    wake_until: Option<u64>,
    screen_on: bool,
}

impl Default for DisplayPowerState {
    fn default() -> Self {
        Self {
            wake_until: None,
            screen_on: true,
        }
    }
}

impl DisplayPowerState {
    pub fn touch(&mut self, night_window: bool, now_seconds: u64) {
        if night_window {
            self.wake_until = Some(now_seconds.saturating_add(NIGHT_WAKE_SECONDS));
        }
    }

    pub fn desired_screen_on(&self, night_window: bool, now_seconds: u64) -> bool {
        !night_window
            || self
                .wake_until
                .is_some_and(|deadline| now_seconds < deadline)
    }

    pub fn reconcile(&mut self, night_window: bool, now_seconds: u64) -> Option<bool> {
        if !night_window {
            self.wake_until = None;
        }

        let desired = self.desired_screen_on(night_window, now_seconds);
        if desired == self.screen_on {
            None
        } else {
            self.screen_on = desired;
            Some(desired)
        }
    }

    #[cfg(test)]
    fn wake_until(&self) -> Option<u64> {
        self.wake_until
    }
}

pub fn apply_screen_power(screen_on: bool) {
    let action = if screen_on { "on" } else { "off" };
    let result = Command::new("xset")
        .args(["dpms", "force", action])
        .status();

    match result {
        Ok(status) if status.success() => {}
        Ok(status) => eprintln!("xset dpms force {action} exited with {status}"),
        Err(error) => eprintln!("unable to run xset dpms force {action}: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touch_wakes_night_screen_for_sixty_seconds() {
        let mut state = DisplayPowerState::default();
        state.touch(true, 1_000);
        assert!(state.desired_screen_on(true, 1_059));
        assert!(!state.desired_screen_on(true, 1_060));
    }

    #[test]
    fn another_touch_extends_the_wake_deadline() {
        let mut state = DisplayPowerState::default();
        state.touch(true, 1_000);
        state.touch(true, 1_040);
        assert!(state.desired_screen_on(true, 1_099));
        assert!(!state.desired_screen_on(true, 1_100));
    }

    #[test]
    fn daytime_always_requests_screen_on_and_clears_wake() {
        let mut state = DisplayPowerState::default();
        state.touch(true, 1_000);
        assert!(state.reconcile(false, 1_010).is_none());
        assert!(state.desired_screen_on(false, 2_000));
        assert!(state.wake_until().is_none());
    }
}
