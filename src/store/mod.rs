use crate::types::Version;
use std::{borrow::Borrow, error::Error, hash::Hash, sync::Arc, time::Duration};
use tokio::time::Instant;

pub mod eviction;
pub mod memory_store;

#[derive(Debug, Clone)]
pub struct KeyRef<K>(pub Arc<K>);

impl<K: Hash> Hash for KeyRef<K> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<K: PartialEq> PartialEq for KeyRef<K> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<K: Eq> Eq for KeyRef<K> {}

impl Borrow<str> for KeyRef<String> {
    #[inline]
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl<K> Borrow<K> for KeyRef<K> {
    #[inline]
    fn borrow(&self) -> &K {
        &self.0
    }
}

pub trait Store: Send + Sync {
    type Key: Hash + Eq + Clone + Send + Sync;
    type State: Send + Sync;
    type Error: Error + Send + Sync + 'static;

    fn update<Q, I, F, R>(
        &self,
        key: &Q,
        now: Instant,
        init: I,
        mutate: F,
    ) -> impl Future<Output = Result<R, Self::Error>> + Send
    where
        KeyRef<Self::Key>: Borrow<Q>,
        Q: ?Sized + Hash + Eq + ToOwned<Owned = Self::Key>,
        R: Send,
        I: FnOnce() -> Self::State + Send,
        F: FnOnce(&mut Self::State) -> R + Send + Sync;

    /// Remove every entry idle for longer than `max_idle`. Returns the count removed.
    /// Only ever called from the background sweep task, never from `check()`.
    fn evict_idle(&self, now: Instant, max_idle: Duration) -> usize;
}

#[derive(Debug, Clone, Copy)]
pub struct Entry<S> {
    pub state: S,
    pub version: Version,
    pub last_accessed: Instant,
}

impl<S> Entry<S>
where
    S: Send + Sync,
{
    pub fn new(state: S, now: Instant) -> Self {
        Entry {
            state,
            version: Version(0),
            last_accessed: now,
        }
    }

    #[inline]
    pub fn touch(&mut self, now: Instant) {
        self.last_accessed = now;
        self.version.0 += 1;
    }
}
