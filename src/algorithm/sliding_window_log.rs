use std::collections::VecDeque;

use tokio::time::Instant;

use super::Algorithm;

use crate::{store::eviction::Eviction, types::Quota};

#[derive(Debug, Clone, Default)]
pub struct SlidingWindowLogState {
    timestamps: VecDeque<Instant>,
}

#[derive(Debug)]
pub struct SlidingWindowLog {
    quota: Quota,
}

impl SlidingWindowLog {
    pub fn new(quota: Quota) -> Self {
        Self { quota }
    }
}

impl Algorithm for SlidingWindowLog {
    type Input = Instant;
    type State = SlidingWindowLogState;

    type Response = bool;

    fn init_state(&self, _: Self::Input) -> Self::State {
        Self::State {
            timestamps: VecDeque::with_capacity(8),
        }
    }

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

        state.timestamps.push_back(input);
        true
    }

    fn eviction(&self, state: &Self::State, now: Instant) -> Eviction {
        let expires_at = match state.timestamps.back() {
            Some(t) => Some(t.clone() + self.quota.get_window()),
            None => None,
        };
        let is_expired = match expires_at {
            Some(expired_at) => expired_at >= now,
            None => true,
        };

        Eviction::new(is_expired, expires_at)
    }
}
