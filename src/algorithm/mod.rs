pub mod fixed_window_counter;
pub mod sliding_window_log;
pub mod state;

pub trait Algorithm {
    type Input: Copy;
    type State: Clone + Default + Send + Sync;
    type Response;

    fn check(&self, state: &mut Self::State, input: Self::Input) -> Self::Response;
}
