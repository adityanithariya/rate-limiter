use std::sync::Arc;

use tokio::time::Instant;

use crate::types::Version;

#[derive(Debug)]
pub struct Eviction {
    is_expired: bool,
    expires_at: Option<Instant>,
}

impl Eviction {
    pub fn new(is_expired: bool, expires_at: Option<Instant>) -> Self {
        Self {
            is_expired,
            expires_at,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.is_expired
    }

    pub fn expires_at(&self) -> Option<Instant> {
        self.expires_at
    }
}

#[derive(Debug)]
pub struct EvictionTask<K> {
    pub key: Arc<K>,
    pub version: Version,
    pub eviction: Eviction,
}
