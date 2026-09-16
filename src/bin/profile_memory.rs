use std::{env, num::NonZeroUsize, time::Duration};

use rate_limiter::{
    algorithm::{
        fixed_window_counter::{FixedWindowCounter, FixedWindowCounterState},
        sliding_window_log::{SlidingWindowLog, SlidingWindowLogState},
    },
    limiter::RateLimiter,
    store::memory_store::InMemoryStore,
    types::Quota,
};
use tokio::time::Instant;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

trait ProfileRunner {
    async fn check_key(&self, key: &str, now: Instant) -> bool;

    async fn run_workload(&self, hits_per_key: usize, keys: &[String]) {
        let now = Instant::now();
        for _ in 0..hits_per_key {
            for key in keys {
                let _ = self.check_key(key, now).await;
            }
        }
    }
}

// Implement for your concrete limiter types
impl ProfileRunner
    for RateLimiter<FixedWindowCounter, InMemoryStore<String, FixedWindowCounterState>>
{
    #[inline]
    async fn check_key(&self, key: &str, now: Instant) -> bool {
        match self.check(key, now).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

impl ProfileRunner for RateLimiter<SlidingWindowLog, InMemoryStore<String, SlidingWindowLogState>> {
    #[inline]
    async fn check_key(&self, key: &str, now: Instant) -> bool {
        match self.check(key, now).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let algo = args.get(1).map(String::as_str).unwrap_or("fixed_window");

    const NUM_KEYS: usize = 50_000;
    const HITS_PER_KEY: usize = 5;

    let keys: Vec<String> = (0..NUM_KEYS).map(|i| format!("user_{i}")).collect();

    println!("Profiling memory for: {algo} ({NUM_KEYS} active keys)...");

    let _profiler = dhat::Profiler::new_heap();

    match algo {
        "fixed_window" => {
            let quota = Quota::new(NonZeroUsize::new(100_000).unwrap(), Duration::from_secs(60));
            let limiter = RateLimiter::new(
                FixedWindowCounter::new(quota),
                InMemoryStore::with_capacity(NUM_KEYS),
            );
            limiter.run_workload(HITS_PER_KEY, &keys).await;

            tokio::time::pause();
            tokio::time::advance(Duration::from_secs(61)).await;
            tokio::task::yield_now().await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        "sliding_window_log" => {
            let quota = Quota::new(NonZeroUsize::new(100_000).unwrap(), Duration::from_secs(60));
            let limiter = RateLimiter::new(
                SlidingWindowLog::new(quota),
                InMemoryStore::with_capacity(NUM_KEYS),
            );
            limiter.run_workload(HITS_PER_KEY, &keys).await;

            tokio::time::pause();
            tokio::time::advance(Duration::from_secs(61)).await;
            tokio::task::yield_now().await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        unknown => {
            eprintln!("Unknown algorithm: {unknown}");
            std::process::exit(1);
        }
    }

    println!("Done. Generating dhat-heap.json...");
}
