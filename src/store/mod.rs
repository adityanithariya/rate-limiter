use crate::types::Version;
use eviction::Eviction;
use std::{borrow::Borrow, error::Error, hash::Hash, sync::Arc};

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
        init: I,
        mutate: F,
    ) -> impl Future<Output = Result<R, Self::Error>> + Send
    where
        KeyRef<Self::Key>: Borrow<Q>,
        Q: ?Sized + Hash + Eq + ToOwned<Owned = Self::Key>,
        R: Send,
        I: FnOnce() -> Self::State + Send,
        F: FnOnce(&mut Entry<Self::State>) -> (R, Eviction) + Send + Sync;

    fn remove_if<Q>(
        &self,
        key: &Q,
        f: impl FnOnce(&KeyRef<Self::Key>, &Entry<Self::State>) -> bool,
    ) -> Option<(KeyRef<Self::Key>, Entry<Self::State>)>
    where
        KeyRef<Self::Key>: Borrow<Q>,
        Q: ?Sized + Hash + Eq + ToOwned<Owned = Self::Key>;
}

#[derive(Debug, Clone, Copy)]
pub struct Entry<S> {
    pub state: S,
    pub version: Version,
}

impl<S> Entry<S>
where
    S: Send + Sync,
{
    pub fn new(state: S) -> Self {
        Entry {
            state,
            version: Version(0),
        }
    }

    pub fn update_version(&mut self) {
        self.version.0 += 1;
    }
}
