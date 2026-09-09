use std::{
    num::NonZeroUsize,
    time::{Duration, Instant},
};

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use pprof::criterion::{Output, PProfProfiler};

// Import your algorithms and limiter
use rate_limiter::{
    algorithm::{
        fixed_window_counter::{FixedWindowCounter, FixedWindowCounterState},
        sliding_window_log::{SlidingWindowLog, SlidingWindowLogState},
    },
    limiter::RateLimiter,
    store::memory_store::InMemoryStore,
    types::Quota,
};

trait LimiterRunner {
    fn check_key<'a>(&self, key: &str, now: Instant) -> bool;
}

impl LimiterRunner
    for RateLimiter<FixedWindowCounter, InMemoryStore<String, FixedWindowCounterState>>
{
    #[inline]
    fn check_key<'a>(&self, key: &str, now: Instant) -> bool {
        self.check(key, now)
    }
}

impl LimiterRunner for RateLimiter<SlidingWindowLog, InMemoryStore<String, SlidingWindowLogState>> {
    #[inline]
    fn check_key<'a>(&self, key: &str, now: Instant) -> bool {
        self.check(key, now)
    }
}

fn build_fixed_window() -> impl LimiterRunner {
    let quota = Quota::new(NonZeroUsize::new(100_000).unwrap(), Duration::from_secs(60));
    RateLimiter::new(FixedWindowCounter::new(quota), InMemoryStore::new())
}

fn build_sliding_window_log() -> impl LimiterRunner {
    let quota = Quota::new(NonZeroUsize::new(100_000).unwrap(), Duration::from_secs(60));
    RateLimiter::new(SlidingWindowLog::new(quota), InMemoryStore::new())
}

fn bench_algorithms(c: &mut Criterion) {
    let mut group = c.benchmark_group("algorithm_comparison");

    // Algorithms under test: (Display Name, Factory Box)
    let algorithms: Vec<(&str, Box<dyn Fn() -> Box<dyn LimiterRunner>>)> = vec![
        ("FixedWindow", Box::new(|| Box::new(build_fixed_window()))),
        (
            "SlidingWindowLog",
            Box::new(|| Box::new(build_sliding_window_log())),
        ),
    ];

    // Workload 1: Contended Single Key
    for (algo_name, factory) in &algorithms {
        group.bench_function(BenchmarkId::new("single_key", algo_name), |b| {
            let limiter = factory();
            let key = "user_static";

            b.iter(|| {
                let res = limiter.check_key(black_box(key), black_box(Instant::now()));
                black_box(res);
            });
        });
    }

    // Workload 2: Multi-Tenant / 1,000 Keys (Measure cache thrashing & hash lookup)
    const KEY_COUNT: usize = 1_000;
    let keys: Vec<String> = (0..KEY_COUNT).map(|i| format!("user_{i}")).collect();

    for (algo_name, factory) in &algorithms {
        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new("1k_keys_round_robin", algo_name), |b| {
            let limiter = factory();
            let mut idx = 0;
            let key = &keys[idx % KEY_COUNT];

            b.iter(|| {
                idx = idx.wrapping_add(1);

                let res = limiter.check_key(black_box(key), black_box(Instant::now()));
                black_box(res);
            });
        });
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = bench_algorithms
}
criterion_main!(benches);
