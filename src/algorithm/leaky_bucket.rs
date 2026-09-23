use crate::{
    algorithm::{
        Algorithm,
        types::{Allowed, RateLimitDecision, Rejected},
    },
    types::{Quota, SecondDuration},
};
use std::{num::NonZeroUsize, time::Duration};
use tokio::time::Instant;

#[derive(Debug, Clone, Copy)]
pub struct LeakyBucketState {
    /// Theoretical Arrival Time: when the bucket will be completely drained
    tat: Instant,
}

#[derive(Debug, Clone, Copy)]
pub struct LeakyBucketCheck {
    pub now: Instant,
    pub cost: NonZeroUsize,
    /// Maximum duration an async caller is permitted to pause before dropping the request
    pub max_wait: SecondDuration,
}

impl LeakyBucketCheck {
    pub fn new(now: Instant, cost: NonZeroUsize, max_wait: SecondDuration) -> Self {
        LeakyBucketCheck {
            now,
            cost,
            max_wait,
        }
    }

    // fn eviction(
    //     &self,
    //     state: &LeakyBucketState,
    //     input: LeakyBucketCheck,
    // ) -> crate::store::eviction::Eviction {
    //     Eviction::new(state.tat >= input.now, Some(state.tat))
    // }
}

impl Default for LeakyBucketCheck {
    fn default() -> Self {
        LeakyBucketCheck {
            now: Instant::now(),
            cost: NonZeroUsize::new(1).unwrap(),
            max_wait: SecondDuration::try_from(Duration::from_secs(10)).unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct LeakyBucket {
    emission_interval: Duration, // Time cost per 1 token (T)
    burst_tolerance: Duration,   // Total bucket depth (tau)
    max_capacity: usize,
}

impl LeakyBucket {
    pub fn new(quota: Quota) -> Self {
        let max_capacity = quota.get_max_capacity();
        let window = quota.get_window();

        // Emission interval = window / capacity
        let emission_interval = window
            .checked_div(max_capacity as u32)
            .unwrap_or(Duration::ZERO);

        Self {
            emission_interval,
            burst_tolerance: window,
            max_capacity,
        }
    }
}

impl Algorithm for LeakyBucket {
    type Input = LeakyBucketCheck;
    type State = LeakyBucketState;
    type Response = RateLimitDecision;

    fn init_state(&self, input: Self::Input) -> Self::State {
        LeakyBucketState { tat: input.now }
    }

    fn check(&self, state: &mut Self::State, input: Self::Input) -> Self::Response {
        let cost = input.cost.get() as u32;

        if cost as usize > self.max_capacity {
            return RateLimitDecision::Rejected(Rejected { retry_after: None });
        }

        let cost_duration = self.emission_interval * cost;
        // The bucket begins draining from either 'now' (if dry) or current 'tat'
        let base_tat = state.tat.max(input.now);
        let new_tat = base_tat + cost_duration;

        // How full is the bucket past 'now'?
        let delay_needed = new_tat.saturating_duration_since(input.now);

        // If delay needed exceeds the bucket depth + max allowable queue wait, drop it
        if delay_needed > self.burst_tolerance + input.max_wait.as_duration() {
            return RateLimitDecision::Rejected(Rejected {
                retry_after: Some(input.now + delay_needed),
            });
        }

        state.tat = new_tat;
        if delay_needed <= self.burst_tolerance {
            RateLimitDecision::Allowed(Allowed { remaining: None })
        } else {
            let wait_duration = delay_needed - self.burst_tolerance;
            RateLimitDecision::WaitUntil(input.now + wait_duration)
        }
    }
}
