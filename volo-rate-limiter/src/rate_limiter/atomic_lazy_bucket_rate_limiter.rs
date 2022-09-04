#[derive(Clone)]
pub struct AtomicLazyBucketRateLimiter(std::sync::Arc<AtomicLazyBucketRateLimiterSharedStatus>);

struct AtomicLazyBucketRateLimiterSharedStatus {
    interval_in_nanos: u64,
    limit: i64,

    last_updated_timestamp_in_nanos: std::sync::atomic::AtomicU64,
    tokens: std::sync::atomic::AtomicI64,
}

impl crate::RateLimiter for AtomicLazyBucketRateLimiter {
    fn new(interval: impl Into<std::time::Duration>, limit: u64) -> Self {
        let limit: i64 = limit.try_into().expect("limit out of range");

        Self(std::sync::Arc::new(
            AtomicLazyBucketRateLimiterSharedStatus {
                interval_in_nanos: interval.into().as_nanos() as u64,
                limit: limit,
                last_updated_timestamp_in_nanos: std::sync::atomic::AtomicU64::new(
                    Self::now_timestamp_in_nanos(),
                ),
                tokens: std::sync::atomic::AtomicI64::new(limit),
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

        if now < last_updated + self.0.interval_in_nanos {
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
                .store(self.0.limit, std::sync::atomic::Ordering::Relaxed);
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

pub type AtomicLazyBucketRateLimiterService<S> =
    crate::RateLimiterService<S, AtomicLazyBucketRateLimiter>;

pub type AtomicLazyBucketRateLimiterLayer = crate::RateLimiterLayer<AtomicLazyBucketRateLimiter>;
