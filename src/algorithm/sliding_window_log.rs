use tokio::time::Instant;

use super::Algorithm;
use crate::{algorithm::types::Now, types::Quota};
use std::collections::VecDeque;

#[derive(Debug, Clone, Default)]
pub struct SlidingWindowLogState {
    timestamps: VecDeque<Instant>,
}

#[derive(Debug)]
pub struct SlidingWindowLog {
    quota: Quota,
}

impl SlidingWindowLog {
    #[inline]
    pub fn new(quota: Quota) -> Self {
        Self { quota }
    }

    // #[inline]
    // fn eviction(&self, state: &SlidingWindowLogState, now: Now) -> Eviction {
    //     let expires_at = match state.timestamps.back() {
    //         Some(t) => Some(t.clone() + self.quota.get_window()),
    //         None => None,
    //     };
    //     let is_expired = match expires_at {
    //         Some(expired_at) => expired_at >= *now,
    //         None => true,
    //     };

    //     Eviction::new(is_expired, expires_at)
    // }
}

impl Algorithm for SlidingWindowLog {
    type Input = Now;
    type State = SlidingWindowLogState;

    type Response = bool;

    #[inline]
    fn init_state(&self, _: Self::Input) -> Self::State {
        Self::State {
            timestamps: VecDeque::with_capacity(8),
        }
    }

    #[inline]
    fn check(&self, state: &mut Self::State, input: Self::Input) -> Self::Response {
        let max_capacity = self.quota.get_max_capacity();

        let threshold = input.checked_sub(self.quota.get_window());

        if let Some(lower_bound) = threshold {
            while let Some(&front) = state.timestamps.front() {
                if front <= lower_bound {
                    state.timestamps.pop_front();
                } else {
                    break;
                }
            }
        }

        if state.timestamps.len() >= max_capacity {
            return false;
        }

        state.timestamps.push_back(*input);
        true
    }
}
