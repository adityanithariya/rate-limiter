use std::{borrow::Borrow, hash::Hash};

pub mod memory_store;

pub trait Store: Send + Sync {
    type Key: Hash + Eq + Send + Sync;
    type State: Default + Send + Sync;

    fn update<Q, F, R>(&self, key: &Q, mutate: F) -> R
    where
        Self::Key: Borrow<Q>,
        Q: ?Sized + Hash + Eq + ToOwned<Owned = Self::Key>,
        F: FnOnce(&mut Self::State) -> R;
}
