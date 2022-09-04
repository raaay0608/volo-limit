pub trait RateLimiter: Send {
    fn new(interval: impl Into<std::time::Duration>, limit: u64) -> Self;

    fn acquire(&self) -> Result<(), ()>;
}

mod atomic_lazy_bucket_rate_limiter;
pub use atomic_lazy_bucket_rate_limiter::*;

mod thread_bucket_rate_limiter;
pub use thread_bucket_rate_limiter::*;

mod tokio_bucket_rate_limiter;
pub use tokio_bucket_rate_limiter::*;
