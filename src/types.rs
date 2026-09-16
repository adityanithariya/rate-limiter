use std::{num::NonZeroUsize, time::Duration};

#[derive(Debug, Clone, Copy)]
pub struct Quota {
    max_capacity: NonZeroUsize,
    window: Duration,
}

impl Quota {
    pub fn new(max_capacity: NonZeroUsize, window: Duration) -> Quota {
        Quota {
            max_capacity,
            window,
        }
    }

    #[inline]
    pub fn get_max_capacity(&self) -> usize {
        self.max_capacity.into()
    }

    #[inline]
    pub fn get_window(&self) -> Duration {
        self.window
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version(pub u64);
