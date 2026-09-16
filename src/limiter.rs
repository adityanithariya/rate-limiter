use std::{borrow::Borrow, hash::Hash};

use crate::{
    algorithm::Algorithm,
    store::{KeyRef, Store},
};

#[derive(Debug)]
pub struct RateLimiter<A, S>
where
    A: Algorithm,
    S: Store,
{
    algorithm: A,
    store: S,
}

impl<A, S> RateLimiter<A, S>
where
    A: Algorithm,
    S: Store<State = A::State> + Send + Sync + 'static,
{
    #[inline]
    pub fn new(algorithm: A, store: S) -> RateLimiter<A, S> {
        RateLimiter { algorithm, store }
    }

    #[inline]
    pub async fn check<K>(&self, key: &K, input: A::Input) -> Result<A::Response, S::Error>
    where
        KeyRef<S::Key>: Borrow<K>,
        K: ?Sized + Hash + Eq + ToOwned<Owned = S::Key>,
    {
        self.store
            .update(
                key,
                || self.algorithm.init_state(input),
                |entry| {
                    entry.update_version();
                    let res = self.algorithm.check(&mut entry.state, input);
                    let eviction = self.algorithm.eviction(&entry.state, input);
                    (res, eviction)
                },
            )
            .await
    }
}
