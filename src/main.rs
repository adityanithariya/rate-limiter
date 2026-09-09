use std::{
    num::NonZeroUsize,
    thread,
    time::{Duration, Instant},
};

use rate_limiter::{
    algorithm::fixed_window_counter::{FixedWindowCounter, FixedWindowCounterState},
    limiter::RateLimiter,
    store::memory_store::InMemoryStore,
    types::Quota,
};

fn main() {
    let fixed_window_counter = FixedWindowCounter::new(Quota::new(
        NonZeroUsize::new(30).unwrap(),
        Duration::from_mins(1),
    ));
    let memory_store: InMemoryStore<&str, FixedWindowCounterState> =
        InMemoryStore::with_capacity(100);
    let rate_limiter = RateLimiter::new(fixed_window_counter, memory_store);

    for i in 0..200 {
        let res = rate_limiter.check(&"user1", Instant::now());
        println!("{i} {res:?}");
        thread::sleep(Duration::from_secs(1));
    }
}
