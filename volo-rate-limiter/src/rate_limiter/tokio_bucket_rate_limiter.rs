#[derive(Clone)]
pub struct TokioBucketRateLimiter {
    status: std::sync::Arc<TokioBucketRateLimiterStatus>,

    // wrapped in `Arc<Mutex>` to satisfy `Clone` and `Send`.
    handle: std::sync::Arc<std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

struct TokioBucketRateLimiterStatus {
    interval: std::time::Duration,
    limit: i64,

    tokens: std::sync::atomic::AtomicI64,

    notify: tokio::sync::Notify,
}

impl crate::RateLimiter for TokioBucketRateLimiter {
    fn new(interval: impl Into<std::time::Duration>, limit: u64) -> Self {
        let limit: i64 = limit.try_into().expect("limit out of range");

        let status = std::sync::Arc::new(TokioBucketRateLimiterStatus {
            interval: interval.into(),
            limit: limit,
            tokens: std::sync::atomic::AtomicI64::new(limit),
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

        // XXX: block_on async funtion in sync function
        futures::executor::block_on(self.handle.lock().unwrap().take().unwrap())
            .expect("joining task panicked");
    }
}

impl TokioBucketRateLimiter {
    async fn proc(status: std::sync::Arc<TokioBucketRateLimiterStatus>) {
        let mut instant = tokio::time::Instant::now();
        loop {
            instant += status.interval;

            tokio::select! {
                _ = status.notify.notified() => {
                    break;
                },
                _ = tokio::time::sleep_until(instant) => {
                    status.tokens.store(status.limit, std::sync::atomic::Ordering::Relaxed);
                },
            }
        }
    }
}

pub type TokioBucketRateLimiterService<S> = crate::RateLimiterService<S, TokioBucketRateLimiter>;

pub type TokioBucketRateLimiterLayer = crate::RateLimiterLayer<TokioBucketRateLimiter>;
