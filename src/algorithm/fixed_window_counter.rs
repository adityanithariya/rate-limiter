use std::time::Instant;

use crate::types::Quota;

use super::Algorithm;

#[derive(Clone, Copy, Debug)]
pub struct FixedWindowCounterState {
    pub count: usize,
    pub window_start: Instant,
}

impl Default for FixedWindowCounterState {
    #[inline]
    fn default() -> Self {
        Self {
            count: 0,
            window_start: Instant::now(),
        }
    }
}

#[derive(Debug)]
pub struct FixedWindowCounter {
    quota: Quota,
}

impl FixedWindowCounter {
    #[inline]
    pub fn new(quota: Quota) -> Self {
        Self { quota }
    }
}

impl Algorithm for FixedWindowCounter {
    type Input = Instant;
    type State = FixedWindowCounterState;
    type Response = bool;

    #[inline]
    fn check(&self, state: &mut Self::State, now: Self::Input) -> Self::Response {
        let window_duration = self.quota.get_window();
        if state.count == 0 || now.saturating_duration_since(state.window_start) >= window_duration
        {
            state.count = 0;
            state.window_start = now;
        }

        let max_capacity = self.quota.get_max_capacity();
        if state.count >= max_capacity {
            return false;
        }

        state.count += 1;
        true
    }
}
