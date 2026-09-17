use std::{num::NonZeroUsize, time::Duration};

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use pprof::criterion::{Output, PProfProfiler};

use rate_limiter::{
    algorithm::{
        Algorithm,
        fixed_window_counter::{FixedWindowCounter, FixedWindowCounterState},
        leaky_bucket::{LeakyBucket, LeakyBucketCheck, LeakyBucketState},
        sliding_window_counter::{SlidingWindowCounter, SlidingWindowCounterState},
        sliding_window_log::{SlidingWindowLog, SlidingWindowLogState},
        token_bucket::{TokenBucket, TokenBucketCheck, TokenBucketState},
    },
    limiter::RateLimiter,
    store::memory_store::InMemoryStore,
    types::{Quota, SecondDuration},
};
use tokio::time::Instant;

// Concrete rate limiter type aliases for cleaner signatures
type FixedWindowLimiter =
    RateLimiter<FixedWindowCounter, InMemoryStore<String, FixedWindowCounterState>>;
type SlidingWindowLogLimiter =
    RateLimiter<SlidingWindowLog, InMemoryStore<String, SlidingWindowLogState>>;
type SlidingWindowCounterLimiter =
    RateLimiter<SlidingWindowCounter, InMemoryStore<String, SlidingWindowCounterState>>;
type TokenBucketLimiter = RateLimiter<TokenBucket, InMemoryStore<String, TokenBucketState>>;
type LeakyBucketLimiter = RateLimiter<LeakyBucket, InMemoryStore<String, LeakyBucketState>>;

fn build_fixed_window() -> FixedWindowLimiter {
    let quota = Quota::new(
        NonZeroUsize::new(100_000).unwrap(),
        SecondDuration::try_from(Duration::from_secs(60)).unwrap(),
    );
    RateLimiter::new(FixedWindowCounter::new(quota), InMemoryStore::new())
}

fn build_sliding_window_log() -> SlidingWindowLogLimiter {
    let quota = Quota::new(
        NonZeroUsize::new(100_000).unwrap(),
        SecondDuration::try_from(Duration::from_secs(60)).unwrap(),
    );
    RateLimiter::new(SlidingWindowLog::new(quota), InMemoryStore::new())
}

fn build_sliding_window_counter() -> SlidingWindowCounterLimiter {
    let quota = Quota::new(
        NonZeroUsize::new(100_000).unwrap(),
        SecondDuration::try_from(Duration::from_secs(60)).unwrap(),
    );
    RateLimiter::new(SlidingWindowCounter::new(quota), InMemoryStore::new())
}

fn build_token_bucket() -> TokenBucketLimiter {
    let quota = Quota::new(
        NonZeroUsize::new(100_000).unwrap(),
        SecondDuration::try_from(Duration::from_secs(60)).unwrap(),
    );
    RateLimiter::new(TokenBucket::new(quota), InMemoryStore::new())
}

fn build_leaky_bucket() -> LeakyBucketLimiter {
    let quota = Quota::new(
        NonZeroUsize::new(100_000).unwrap(),
        SecondDuration::try_from(Duration::from_secs(60)).unwrap(),
    );
    RateLimiter::new(LeakyBucket::new(quota), InMemoryStore::new())
}

/// Generic static-dispatch benchmark runner.
/// Zero dynamic dispatch, zero heap allocation (no Box::pin), pure inlined futures.
fn bench_limiter_workloads<A, F>(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    runtime: &tokio::runtime::Runtime,
    algo_name: &str,
    factory: F,
    static_now: A::Input,
    keys: &[String],
) where
    A: Algorithm + 'static,
    A::State: 'static,
    F: Fn() -> RateLimiter<A, InMemoryStore<String, A::State>>,
{
    // Workload 1: Contended Single Key
    group.bench_function(BenchmarkId::new("single_key", algo_name), |b| {
        let limiter = runtime.block_on(async { factory() });
        let key = "user_static";

        b.to_async(runtime).iter(|| async {
            let res = limiter
                .check::<str>(black_box(key), black_box(static_now))
                .await;
            let _ = black_box(res);
        });
    });

    // Workload 2: Multi-Tenant / 1,000 Keys Round-Robin
    group.bench_function(BenchmarkId::new("1k_keys_round_robin", algo_name), |b| {
        let limiter = runtime.block_on(async { factory() });
        let mut idx = 0usize;
        let key_count = keys.len();

        b.to_async(runtime).iter(|| {
            let key = &keys[idx % key_count];
            idx = idx.wrapping_add(1);

            async {
                let res = limiter
                    .check::<str>(black_box(key.as_str()), black_box(static_now))
                    .await;
                let _ = black_box(res);
            }
        });
    });
}

fn bench_algorithms(c: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime");

    let mut group = c.benchmark_group("algorithm_comparison");
    group.throughput(Throughput::Elements(1));

    let static_now = Instant::now();

    const KEY_COUNT: usize = 1_000;
    let keys: Vec<String> = (0..KEY_COUNT).map(|i| format!("user_{i}")).collect();

    // 1. Benchmark FixedWindowCounter with zero allocations
    bench_limiter_workloads(
        &mut group,
        &runtime,
        "FixedWindow",
        build_fixed_window,
        static_now,
        &keys,
    );

    // 2. Benchmark SlidingWindowLog with zero allocations
    bench_limiter_workloads(
        &mut group,
        &runtime,
        "SlidingWindowLog",
        build_sliding_window_log,
        static_now,
        &keys,
    );

    bench_limiter_workloads(
        &mut group,
        &runtime,
        "SlidingWindowCounter",
        build_sliding_window_counter,
        static_now,
        &keys,
    );

    bench_limiter_workloads(
        &mut group,
        &runtime,
        "TokenBucket",
        build_token_bucket,
        TokenBucketCheck::new(static_now, NonZeroUsize::new(1).unwrap()),
        &keys,
    );

    bench_limiter_workloads(
        &mut group,
        &runtime,
        "LeakyBucket",
        build_leaky_bucket,
        LeakyBucketCheck::new(
            static_now,
            NonZeroUsize::new(1).unwrap(),
            SecondDuration::try_from(Duration::from_secs(10)).unwrap(),
        ),
        &keys,
    );

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = bench_algorithms
}
criterion_main!(benches);
