/// The interface definition of a rate limiter.
pub trait RateLimiter: Send {
    /// Creates a `RateLimiter` with limit duration and limit quota.
    fn new(duration: impl Into<std::time::Duration>, quota: u64) -> Self;

    /// Try to acquire a request quota.
    ///
    /// If the request is determined to be passed, the method returns `Ok(())`, otherwise returns `Err(())`
    fn acquire(&self) -> Result<(), ()>;
}

mod atomic_lazy_bucket_rate_limiter;
pub use atomic_lazy_bucket_rate_limiter::*;

mod thread_bucket_rate_limiter;
pub use thread_bucket_rate_limiter::*;

#[cfg(feature = "tokio")]
mod tokio_bucket_rate_limiter;
#[cfg(feature = "tokio")]
pub use tokio_bucket_rate_limiter::*;
