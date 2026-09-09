use std::{borrow::Borrow, hash::Hash};

use dashmap::DashMap;

use crate::store::Store;

#[derive(Debug)]
pub struct InMemoryStore<K, V>
where
    K: Hash + Eq + Send + Sync,
    V: Clone + Send + Sync,
{
    data: DashMap<K, V>,
}

impl<K, V> InMemoryStore<K, V>
where
    K: Hash + Eq + Send + Sync,
    V: Clone + Send + Sync,
{
    #[inline]
    pub fn new() -> Self {
        Self {
            data: DashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: DashMap::with_capacity(capacity),
        }
    }
}

impl<K, V> Default for InMemoryStore<K, V>
where
    K: Hash + Clone + Eq + Send + Sync,
    V: Clone + Default + Send + Sync,
{
    fn default() -> Self {
        Self {
            data: DashMap::default(),
        }
    }
}

impl<K, V> Store for InMemoryStore<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Default + Send + Sync + 'static,
{
    type Key = K;
    type State = V;

    #[inline]
    fn update<Q, F, R>(&self, key: &Q, mutate: F) -> R
    where
        Self::Key: Borrow<Q>,
        Q: ?Sized + Hash + Eq + ToOwned<Owned = Self::Key>,
        F: FnOnce(&mut Self::State) -> R,
    {
        if let Some(mut state) = self.data.get_mut(key) {
            return mutate(state.value_mut());
        }

        let mut state = self.data.entry(key.to_owned()).or_default();
        mutate(state.value_mut())
    }
}
