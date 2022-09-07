/// `ThreadBucketRateLimiter` is a bucket rate limiter implementation,
/// using a dedicated tokio task as token producer.
///
/// This rate limiter implementation requires the server using tokio as runtime.
#[derive(Clone)]
pub struct TokioBucketRateLimiter {
    status: std::sync::Arc<TokioBucketRateLimiterStatus>,

    // wrapped in `Arc<Mutex<...>>` to satisfy `Clone` and `Send` requirements.
    handle: std::sync::Arc<std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

struct TokioBucketRateLimiterStatus {
    duration: std::time::Duration,
    quota: i64,

    tokens: std::sync::atomic::AtomicI64,

    notify: tokio::sync::Notify,
}

impl crate::RateLimiter for TokioBucketRateLimiter {
    fn new(duration: impl Into<std::time::Duration>, quota: u64) -> Self {
        let quota: i64 = quota.try_into().expect("limit quota out of range");

        let status = std::sync::Arc::new(TokioBucketRateLimiterStatus {
            duration: duration.into(),
            quota: quota,
            tokens: std::sync::atomic::AtomicI64::new(quota),
            notify: tokio::sync::Notify::new(),
        });

        let _status = status.clone();
        let handle = tokio::spawn(async move {
            TokioBucketRateLimiter::proc(_status).await;
        });

        Self {
            status,
            handle: std::sync::Arc::new(std::sync::Mutex::new(Some(handle))),
        }
    }

    fn acquire(&self) -> Result<(), ()> {
        match self
            .status
            .tokens
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed)
        {
            1.. => Ok(()),
            _ => {
                self.status
                    .tokens
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Err(())
            }
        }
    }
}

impl Drop for TokioBucketRateLimiter {
    fn drop(&mut self) {
        self.status.notify.notify_one();

        // XXX: block on async funtion in sync function
        futures::executor::block_on(self.handle.lock().unwrap().take().unwrap())
            .expect("joining task panicked");
    }
}

impl TokioBucketRateLimiter {
    async fn proc(status: std::sync::Arc<TokioBucketRateLimiterStatus>) {
        let mut instant = tokio::time::Instant::now();
        loop {
            instant += status.duration;

            tokio::select! {
                _ = status.notify.notified() => {
                    break;
                },
                _ = tokio::time::sleep_until(instant) => {
                    status.tokens.store(status.quota, std::sync::atomic::Ordering::Relaxed);
                },
            }
        }
    }
}

/// `RateLimiterService` with `TokioBucketRateLimiter` as its internal limiter implementation.
pub type TokioBucketRateLimiterService<S> = crate::RateLimiterService<S, TokioBucketRateLimiter>;

/// The `volo::layer` implementation of `RateLimiterService` with `TokioBucketRateLimiter` as its internal limiter implementation.
pub type TokioBucketRateLimiterLayer = crate::RateLimiterLayer<TokioBucketRateLimiter>;
