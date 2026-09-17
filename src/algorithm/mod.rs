use crate::store::eviction;

pub mod fixed_window_counter;
pub mod leaky_bucket;
pub mod sliding_window_counter;
pub mod sliding_window_log;
pub mod token_bucket;
pub mod types;

pub trait Algorithm: Send + Sync {
    type Input: Copy + Send + Sync;
    type State: Clone + Send + Sync;
    type Response: Send;

    fn init_state(&self, input: Self::Input) -> Self::State;
    fn check(&self, state: &mut Self::State, input: Self::Input) -> Self::Response;
    fn eviction(&self, state: &Self::State, input: Self::Input) -> eviction::Eviction;
}
