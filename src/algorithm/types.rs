use std::{
    ops::{Add, Deref, DerefMut, Sub},
    time::Duration,
};
use tokio::time::Instant;

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Now(Instant);

impl Default for Now {
    fn default() -> Self {
        Now(Instant::now())
    }
}

impl Deref for Now {
    type Target = Instant;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Now {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Instant> for Now {
    fn from(value: Instant) -> Self {
        Now(value)
    }
}

impl From<Now> for Instant {
    fn from(time: Now) -> Self {
        time.0
    }
}

impl Add<Duration> for Now {
    type Output = Now;

    fn add(self, rhs: Duration) -> Self::Output {
        Now(self.0 + rhs)
    }
}

impl Sub<Duration> for Now {
    type Output = Now;

    fn sub(self, rhs: Duration) -> Self::Output {
        Now(self.0 - rhs)
    }
}

impl Sub<Now> for Now {
    type Output = Duration;

    fn sub(self, rhs: Now) -> Self::Output {
        self.0 - rhs.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Allowed {
    pub remaining: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
pub struct Rejected {
    pub retry_after: Option<Instant>,
}

#[derive(Debug)]
pub enum RateLimitDecision {
    Allowed(Allowed),
    Rejected(Rejected),
    WaitUntil(Instant),
}
