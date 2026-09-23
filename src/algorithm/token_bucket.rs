use super::Algorithm;
use crate::{
    algorithm::types::{Allowed, RateLimitDecision, Rejected},
    types::Quota,
};
use std::{num::NonZeroUsize, time::Duration};
use tokio::time::Instant;

#[derive(Debug, Clone, Copy)]
pub struct TokenBucketState {
    last_request_time: Instant,
    tokens: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct TokenBucketCheck {
    now: Instant,
    cost: NonZeroUsize,
}

impl TokenBucketCheck {
    #[inline]
    pub fn new(now: Instant, cost: NonZeroUsize) -> Self {
        Self { now, cost }
    }
}

impl Default for TokenBucketCheck {
    fn default() -> Self {
        Self {
            now: Instant::now(),
            cost: NonZeroUsize::new(1).unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct TokenBucket {
    refill_rate: f64,
    emission_interval: f64,
    max_capacity: f64,
}

impl TokenBucket {
    #[inline]
    pub fn new(quota: Quota) -> Self {
        let max_capacity = quota.get_max_capacity() as f64;
        let window_ns = quota.get_window().as_nanos() as f64;
        Self {
            refill_rate: max_capacity / window_ns,
            emission_interval: window_ns / max_capacity,
            max_capacity: quota.get_max_capacity() as f64,
        }
    }

    fn refill_tokens(&self, state: &mut TokenBucketState, input: &TokenBucketCheck) {
        if state.tokens < self.max_capacity {
            let elapsed_ns = input
                .now
                .saturating_duration_since(state.last_request_time)
                .as_nanos() as f64;
            if elapsed_ns > 0.0 {
                let refilled_tokens = state.tokens + elapsed_ns * self.refill_rate;
                state.tokens = refilled_tokens.min(self.max_capacity);
                state.last_request_time = input.now;
            }
        } else {
            state.last_request_time = input.now;
        }
    }

    #[inline(always)]
    fn duration_for_tokens(&self, missing_tokens: f64) -> Duration {
        if missing_tokens <= 0.0 || !self.emission_interval.is_finite() {
            return Duration::ZERO;
        }
        let nanos = (missing_tokens * self.emission_interval) as u64;
        Duration::from_nanos(nanos)
    }

    // #[inline]
    // fn eviction(
    //     &self,
    //     state: &TokenBucketState,
    //     input: TokenBucketCheck,
    // ) -> crate::store::eviction::Eviction {
    //     // let mut new_state = state.clone();
    //     // self.refill_tokens(&mut new_state, &input);
    //     // if new_state.tokens == self.max_capacity {
    //     //     return Eviction::new(true, None);
    //     // }

    //     let missing_tokens = self.max_capacity - state.tokens;
    //     let refill_ns = missing_tokens / self.refill_rate;
    //     let refill_duration = if refill_ns.is_finite() {
    //         Duration::from_nanos(refill_ns as u64)
    //     } else {
    //         Duration::ZERO
    //     };

    //     Eviction::new(
    //         missing_tokens.abs() < f64::EPSILON,
    //         Some(input.now + refill_duration),
    //     )
    // }
}

impl Algorithm for TokenBucket {
    type Input = TokenBucketCheck;
    type State = TokenBucketState;
    type Response = RateLimitDecision;

    #[inline]
    fn init_state(&self, input: Self::Input) -> Self::State {
        TokenBucketState {
            last_request_time: input.now,
            tokens: self.max_capacity,
        }
    }

    #[inline]
    fn check(&self, state: &mut Self::State, input: Self::Input) -> Self::Response {
        let cost = input.cost.get() as f64;
        if cost > self.max_capacity {
            return RateLimitDecision::Rejected(Rejected { retry_after: None });
        }

        self.refill_tokens(state, &input);

        if state.tokens < cost {
            let deficit = cost - state.tokens;
            let retry_duration = self.duration_for_tokens(deficit);

            return RateLimitDecision::Rejected(Rejected {
                retry_after: Some(input.now + retry_duration),
            });
        }

        state.tokens -= cost;
        return RateLimitDecision::Allowed(Allowed {
            remaining: Some(state.tokens as usize),
        });
    }
}
