use std::{borrow::Borrow, hash::Hash};

use crate::{algorithm::Algorithm, store::Store};

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
    S: Store<State = A::State>,
{
    pub fn new(algorithm: A, store: S) -> RateLimiter<A, S> {
        RateLimiter { algorithm, store }
    }

    pub fn check<K>(&self, key: &K, input: A::Input) -> A::Response
    where
        S::Key: Borrow<K>,
        K: ?Sized + Hash + Eq + ToOwned<Owned = S::Key>,
    {
        self.store
            .update(key, |state| self.algorithm.check(state, input))
    }
}
