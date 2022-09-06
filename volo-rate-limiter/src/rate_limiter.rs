// The trait of a rate limiter.
pub trait RateLimiter: Send {
    /// Creates a `RateLimiter` with limit interval and limit quota.
    fn new(interval: impl Into<std::time::Duration>, limit: u64) -> Self;

    /// Try to acquire a request quota.
    /// If the request is determined to be passed, the method returns `Ok(())`, otherwise returns `Err(())`
    fn acquire(&self) -> Result<(), ()>;
}

mod atomic_lazy_bucket_rate_limiter;
pub use atomic_lazy_bucket_rate_limiter::*;

mod thread_bucket_rate_limiter;
pub use thread_bucket_rate_limiter::*;

mod tokio_bucket_rate_limiter;
pub use tokio_bucket_rate_limiter::*;
