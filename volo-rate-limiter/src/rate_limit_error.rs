/// The error type returned by [RateLimiterService](crate::RateLimiterService).
#[derive(Debug)]
pub struct RateLimiterError;

impl std::fmt::Display for RateLimiterError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "rate limited")
    }
}

impl std::error::Error for RateLimiterError {}
