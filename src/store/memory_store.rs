use super::eviction::{Eviction, EvictionTask};
use super::{Entry, KeyRef, Store};
use crate::types::Version;
use ahash::RandomState;
use dashmap::DashMap;
use futures_util::StreamExt;
use std::{borrow::Borrow, hash::Hash, sync::Arc};
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::time::Instant;
use tokio_util::time::DelayQueue;

#[derive(Debug)]
pub struct InMemoryStore<K, V>
where
    K: Hash + Eq + Send + Sync,
    V: Send + Sync,
{
    data: Arc<DashMap<KeyRef<K>, Entry<V>, RandomState>>,
    tx: mpsc::Sender<EvictionTask<K>>,
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
        let data = Arc::new(DashMap::with_capacity_and_hasher(
            capacity,
            RandomState::new(),
        ));
        let (tx, mut rx) = mpsc::channel::<EvictionTask<K>>(1024);

        let store_data = Arc::clone(&data);
        tokio::spawn(async move {
            let mut queue = DelayQueue::<(Arc<K>, Version)>::with_capacity(capacity);

            loop {
                tokio::select! {
                    biased;

                    Some(task) = rx.recv() => {
                        if let Some(expired_at) = task.eviction.expires_at() {
                            let duration = expired_at.saturating_duration_since(Instant::now());
                            queue.insert((task.key, task.version), duration);
                        }
                    }
                    Some(expired) = queue.next(), if !queue.is_empty() => {
                        let (key, expired_version) = expired.into_inner();

                        store_data.remove_if(&*key, |_k, entry: &Entry<V>| {
                            entry.version == expired_version
                        });
                    }
                    else => break,
                }
            }
        });

        Self { data, tx }
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
        init: I,
        mutate: F,
    ) -> impl Future<Output = Result<R, InMemoryStoreError>> + Send
    where
        KeyRef<Self::Key>: Borrow<Q>,
        Q: ?Sized + Hash + Eq + ToOwned<Owned = Self::Key>,
        R: Send,
        I: FnOnce() -> Self::State + Send,
        F: FnOnce(&mut Entry<Self::State>) -> (R, Eviction) + Send + Sync,
    {
        let (res, version, eviction, key_arc) = if let Some(mut state) = self.data.get_mut(key) {
            let res = mutate(&mut state.value_mut());
            let task_key = Arc::clone(&state.key().0);
            (res.0, state.value().version, res.1, task_key)
        } else {
            let key_arc = Arc::new(key.to_owned());
            let mut state = self
                .data
                .entry(KeyRef(Arc::clone(&key_arc)))
                .or_insert_with(|| Entry::new(init()));
            let res = mutate(&mut state.value_mut());
            (res.0, state.value().version, res.1, key_arc)
        };
        if eviction.expires_at().is_some() {
            let task = EvictionTask {
                key: key_arc,
                version,
                eviction,
            };
            let _ = self.tx.try_send(task);
        }
        std::future::ready(Ok(res))
    }

    fn remove_if<Q>(
        &self,
        key: &Q,
        f: impl FnOnce(&KeyRef<Self::Key>, &Entry<V>) -> bool,
    ) -> Option<(KeyRef<Self::Key>, Entry<V>)>
    where
        KeyRef<Self::Key>: Borrow<Q>,
        Q: ?Sized + Hash + Eq + ToOwned<Owned = Self::Key>,
    {
        self.data.remove_if(key, f)
    }
}
