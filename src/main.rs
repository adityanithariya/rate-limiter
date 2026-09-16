use std::{num::NonZeroUsize, sync::Arc, thread, time::Duration};

use rate_limiter::{
    algorithm::fixed_window_counter::{FixedWindowCounter, FixedWindowCounterState},
    limiter::RateLimiter,
    store::memory_store::InMemoryStore,
    types::Quota,
};
use tokio::{
    task::JoinSet,
    time::{Instant, sleep},
};

#[tokio::main]
async fn main() {
    let fixed_window_counter = FixedWindowCounter::new(Quota::new(
        NonZeroUsize::new(30).unwrap(),
        Duration::from_mins(1),
    ));
    let memory_store: InMemoryStore<String, FixedWindowCounterState> =
        InMemoryStore::with_capacity(100);
    let rate_limiter = Arc::new(RateLimiter::new(fixed_window_counter, memory_store));

    let mut set = JoinSet::new();
    for i in 0..200 {
        let rlm = Arc::clone(&rate_limiter);
        set.spawn(async move {
            let res = rlm.check("user1", Instant::now()).await;
            match res {
                Ok(check) => println!("{i} {check:?}"),
                Err(err) => println!("Error: {err:?}"),
            }
            thread::sleep(Duration::from_secs(1));
        });
        sleep(Duration::from_millis(100)).await;
    }

    while let Some(res) = set.join_next().await {
        if let Err(err) = res {
            eprintln!("Task failed or panicked: {err}");
        }
    }
}
