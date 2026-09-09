use std::{collections::VecDeque, time::Instant};

use crate::{algorithm::Algorithm, types::Quota};

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
}
