use tokio::time::Instant;

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
