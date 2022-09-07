/// A bucket rate limiter implementation bases on lazy-update strategy.
///
/// The operations are lock-free and based on atomic CAS operations.
///
/// # Notice
///
/// Note that this implementation does not provide a precise limitation.
///
/// On our tests, this limiter may allow slightly fewer requests to pass (95% - 99% was observed) under normal circumstances,
/// and may allow quite fewer requests to pass (70% - 90% was observed) if server is overloaded.
///
/// This limiter is also observed to allow quite fewer requests to pass if request flow is uneven.
#[derive(Clone)]
pub struct AtomicLazyBucketRateLimiter(std::sync::Arc<AtomicLazyBucketRateLimiterSharedStatus>);

struct AtomicLazyBucketRateLimiterSharedStatus {
    duration_in_nanos: u64,
    quota: i64,

    last_updated_timestamp_in_nanos: std::sync::atomic::AtomicU64,
    tokens: std::sync::atomic::AtomicI64,
}

impl crate::RateLimiter for AtomicLazyBucketRateLimiter {
    fn new(duration: impl Into<std::time::Duration>, quota: u64) -> Self {
        let quota: i64 = quota.try_into().expect("limit quota out of range");

        Self(std::sync::Arc::new(
            AtomicLazyBucketRateLimiterSharedStatus {
                duration_in_nanos: duration.into().as_nanos() as u64,
                quota: quota,
                last_updated_timestamp_in_nanos: std::sync::atomic::AtomicU64::new(
                    Self::now_timestamp_in_nanos(),
                ),
                tokens: std::sync::atomic::AtomicI64::new(quota),
            },
        ))
    }

    fn acquire(&self) -> Result<(), ()> {
        self.fill_tokens();
        self.do_acquire()
    }
}

impl AtomicLazyBucketRateLimiter {
    fn fill_tokens(&self) {
        let now = Self::now_timestamp_in_nanos();
        let last_updated = self
            .0
            .last_updated_timestamp_in_nanos
            .load(std::sync::atomic::Ordering::Relaxed);

        if now < last_updated + self.0.duration_in_nanos {
            return;
        }

        if let Ok(_) = self.0.last_updated_timestamp_in_nanos.compare_exchange(
            last_updated,
            now,
            std::sync::atomic::Ordering::Relaxed,
            std::sync::atomic::Ordering::Relaxed,
        ) {
            self.0
                .tokens
                .store(self.0.quota, std::sync::atomic::Ordering::Relaxed);
        }
    }

    fn do_acquire(&self) -> Result<(), ()> {
        match self
            .0
            .tokens
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed)
        {
            1.. => Ok(()),
            _ => {
                self.0
                    .tokens
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Err(())
            }
        }
    }

    fn now_timestamp_in_nanos() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
}

/// A [RateLimiterService](crate::RateLimiterService) with [AtomicLazyBucketRateLimiter]
/// as its internal rate limiter implementation.
pub type AtomicLazyBucketRateLimiterService<S> =
    crate::RateLimiterService<S, AtomicLazyBucketRateLimiter>;

/// The [volo::Layer] implementation of [RateLimiterService](crate::RateLimiterService)
/// with [AtomicLazyBucketRateLimiter] as its internal rate limiter implementation.
pub type AtomicLazyBucketRateLimiterLayer = crate::RateLimiterLayer<AtomicLazyBucketRateLimiter>;
