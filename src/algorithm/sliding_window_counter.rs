use tokio::time::Instant;

use super::{Algorithm, types::Now};
use crate::types::Quota;

#[derive(Clone, Copy, Debug)]
pub struct SlidingWindowCounterState {
    last_window_count: usize,
    count: usize,
    window_start: Instant,
}

#[derive(Debug)]
pub struct SlidingWindowCounter {
    quota: Quota,
}

impl SlidingWindowCounter {
    #[inline]
    pub fn new(quota: Quota) -> Self {
        Self { quota }
    }

    // #[inline]
    // fn eviction(
    //     &self,
    //     state: &SlidingWindowCounterState,
    //     now: Now,
    // ) -> crate::store::eviction::Eviction {
    //     let expires_at = state.window_start + (self.quota.get_window() * 2);
    //     let is_expired = *now >= expires_at;

    //     Eviction::new(is_expired, Some(expires_at))
    // }
}

impl Algorithm for SlidingWindowCounter {
    type Input = Now;
    type State = SlidingWindowCounterState;
    type Response = bool;

    #[inline]
    fn init_state(&self, now: Self::Input) -> Self::State {
        SlidingWindowCounterState {
            last_window_count: 0,
            count: 0,
            window_start: *now,
        }
    }

    #[inline]
    fn check(&self, state: &mut Self::State, now: Self::Input) -> Self::Response {
        let time_elapsed = now.saturating_duration_since(state.window_start);
        let window = self.quota.get_window();
        if time_elapsed >= window {
            state.last_window_count = if time_elapsed < window * 2 {
                state.count
            } else {
                0
            };
            state.count = 0;
            state.window_start = *now;
        }
        let max_capacity = self.quota.get_max_capacity();
        if state.count > max_capacity {
            return false;
        }

        let progress = time_elapsed.as_nanos().saturating_div(window.as_nanos());
        let weight = (state.last_window_count as u128 * (1 - progress)) as usize;
        let estimated_count = state.count + weight;

        if estimated_count >= max_capacity {
            return false;
        }

        state.count += 1;
        true
    }
}
