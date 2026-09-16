use tokio::time::Instant;

use crate::{store::eviction::Eviction, types::Quota};

use super::Algorithm;

#[derive(Clone, Copy, Debug)]
pub struct FixedWindowCounterState {
    count: usize,
    window_start: Instant,
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

    fn init_state(&self, now: Self::Input) -> Self::State {
        Self::State {
            count: 0,
            window_start: now,
        }
    }

    #[inline]
    fn check(&self, state: &mut Self::State, now: Self::Input) -> Self::Response {
        let window_duration = self.quota.get_window();
        if state.count == 0 || now.saturating_duration_since(state.window_start) >= window_duration
        {
            state.count = 0;
            state.window_start = now;
        }

        if state.count >= self.quota.get_max_capacity() {
            return false;
        }

        state.count += 1;
        true
    }

    fn eviction(&self, state: &Self::State, now: Instant) -> Eviction {
        let expires_at = state.window_start + self.quota.get_window();
        let is_expired = expires_at >= now;

        Eviction::new(is_expired, Some(expires_at))
    }
}
