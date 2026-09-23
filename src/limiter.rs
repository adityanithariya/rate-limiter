use crate::{
    algorithm::Algorithm,
    store::{KeyRef, Store},
    types::SecondDuration,
};
use std::{borrow::Borrow, hash::Hash, sync::Arc};
use tokio::time::Instant;

#[derive(Debug)]
pub struct RateLimiter<A, S>
where
    A: Algorithm,
    S: Store,
{
    algorithm: Arc<A>,
    store: Arc<S>,
}

impl<A, S> RateLimiter<A, S>
where
    A: Algorithm + Send + Sync + 'static,
    S: Store<State = A::State> + Send + Sync + 'static,
{
    /// `max_idle` should be at least as long as the algorithm's window (a bucket
    /// that hasn't refilled yet shouldn't be swept). `sweep_interval` controls
    /// how eagerly memory is reclaimed vs. how much background work runs.
    #[inline]
    pub fn new(
        algorithm: A,
        store: S,
        max_idle: SecondDuration,
        sweep_interval: SecondDuration,
    ) -> RateLimiter<A, S> {
        let store_arc = Arc::new(store);
        let algo_arc = Arc::new(algorithm);

        let store_bg = Arc::clone(&store_arc);
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(sweep_interval.as_duration());
            loop {
                ticker.tick().await;
                let removed = store_bg.evict_idle(Instant::now(), max_idle.as_duration());
                if removed > 0 {
                    tracing::debug!(removed, "swept idle rate-limit entries");
                }
            }
        });

        RateLimiter {
            algorithm: algo_arc,
            store: store_arc,
        }
    }

    #[inline]
    pub async fn check<K>(&self, key: &K, input: A::Input) -> Result<A::Response, S::Error>
    where
        KeyRef<S::Key>: Borrow<K>,
        K: ?Sized + Hash + Eq + ToOwned<Owned = S::Key>,
    {
        let now = Instant::now();
        self.store
            .update(
                key,
                now,
                || self.algorithm.init_state(input),
                |state| self.algorithm.check(state, input),
            )
            .await
    }
}
