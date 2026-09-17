use std::{
    num::{NonZeroU64, NonZeroUsize},
    time::Duration,
};

/// A validated duration that is always an integer number of seconds >= 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SecondDuration(NonZeroU64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DurationError {
    Zero,
    HasSubsecondPrecision,
}

impl SecondDuration {
    pub const fn from_secs(secs: NonZeroU64) -> Self {
        Self(secs)
    }

    #[inline]
    pub const fn as_duration(&self) -> Duration {
        Duration::from_secs(self.0.get())
    }

    #[inline]
    pub const fn as_secs(&self) -> u64 {
        self.0.get()
    }
}

impl TryFrom<Duration> for SecondDuration {
    type Error = DurationError;

    fn try_from(duration: Duration) -> Result<Self, Self::Error> {
        if duration.is_zero() {
            return Err(DurationError::Zero);
        }
        if duration.subsec_nanos() != 0 {
            return Err(DurationError::HasSubsecondPrecision);
        }

        NonZeroU64::new(duration.as_secs())
            .map(SecondDuration)
            .ok_or(DurationError::Zero)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Quota {
    max_capacity: NonZeroUsize,
    window: SecondDuration,
}

impl Quota {
    pub fn new(max_capacity: NonZeroUsize, window: SecondDuration) -> Quota {
        Quota {
            max_capacity,
            window,
        }
    }

    #[inline]
    pub fn get_max_capacity(&self) -> usize {
        self.max_capacity.get()
    }

    #[inline]
    pub fn get_window(&self) -> Duration {
        self.window.as_duration()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version(pub u64);
