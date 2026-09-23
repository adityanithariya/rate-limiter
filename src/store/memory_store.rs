use super::{Entry, KeyRef, Store};
use ahash::RandomState;
use dashmap::DashMap;
use std::time::Duration;
use std::{borrow::Borrow, hash::Hash};
use thiserror::Error;
use tokio::time::Instant;

#[derive(Debug)]
pub struct InMemoryStore<K, V>
where
    K: Hash + Eq + Send + Sync,
    V: Send + Sync,
{
    data: DashMap<KeyRef<K>, Entry<V>, RandomState>,
}

impl<K, V> InMemoryStore<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(64)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: DashMap::with_capacity_and_hasher(capacity, RandomState::new()),
        }
    }
}

impl<K, V> Default for InMemoryStore<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Drop for InMemoryStore<K, V>
where
    K: Hash + Eq + Send + Sync,
    V: Send + Sync,
{
    fn drop(&mut self) {
        self.data.clear();
        self.data.shrink_to_fit();
    }
}

#[derive(Error, Debug)]
pub enum InMemoryStoreError {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl<K, V> Store for InMemoryStore<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    KeyRef<K>: Borrow<K>,
{
    type Key = K;
    type State = V;
    type Error = InMemoryStoreError;

    #[inline]
    fn update<Q, I, F, R>(
        &self,
        key: &Q,
        now: Instant,
        init: I,
        mutate: F,
    ) -> impl Future<Output = Result<R, InMemoryStoreError>> + Send
    where
        KeyRef<Self::Key>: Borrow<Q>,
        Q: ?Sized + Hash + Eq + ToOwned<Owned = Self::Key>,
        R: Send,
        I: FnOnce() -> Self::State + Send,
        F: FnOnce(&mut Self::State) -> R + Send + Sync,
    {
        let res = match self.data.get_mut(key) {
            Some(mut entry) => {
                let r = mutate(&mut entry.value_mut().state);
                entry.value_mut().touch(now);
                r
            }
            None => {
                let mut entry = self
                    .data
                    .entry(KeyRef(std::sync::Arc::new(key.to_owned())))
                    .or_insert_with(|| Entry::new(init(), now));
                let r = mutate(&mut entry.value_mut().state);
                entry.value_mut().touch(now);
                r
            }
        };
        std::future::ready(Ok(res))
    }
    fn evict_idle(&self, now: Instant, max_idle: Duration) -> usize {
        let before = self.data.len();
        self.data
            .retain(|_, entry| now.saturating_duration_since(entry.last_accessed) < max_idle);
        before - self.data.len()
    }
}
