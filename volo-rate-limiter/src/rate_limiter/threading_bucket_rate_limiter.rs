/// A bucket rate limiter implementation, using a dedicated [std::thread::Thread] as token producer.
#[derive(Clone)]
pub struct ThreadingBucketRateLimiter {
    status: std::sync::Arc<ThreadingBucketRateLimiterStatus>,

    // wrapped in `Arc<Mutex<...>>` to satisfy `Clone` and `Send` requirements.
    handle: std::sync::Arc<std::sync::Mutex<Option<std::thread::JoinHandle<()>>>>,
}

struct ThreadingBucketRateLimiterStatus {
    duration: std::time::Duration,
    quota: i64,

    tokens: std::sync::atomic::AtomicI64,

    // `tx` and `rx` are used as thread termination notifier.
    tx: std::sync::Mutex<std::sync::mpsc::Sender<()>>,
    rx: std::sync::Mutex<std::sync::mpsc::Receiver<()>>,
}

impl crate::RateLimiter for ThreadingBucketRateLimiter {
    fn new(duration: impl Into<std::time::Duration>, quota: u64) -> Self {
        let quota: i64 = quota.try_into().expect("limit quota out of range");

        let (tx, rx) = std::sync::mpsc::channel();

        let status = std::sync::Arc::new(ThreadingBucketRateLimiterStatus {
            duration: duration.into(),
            quota: quota,
            tokens: std::sync::atomic::AtomicI64::new(quota),
            tx: std::sync::Mutex::new(tx),
            rx: std::sync::Mutex::new(rx),
        });

        let _status = status.clone();
        let handle = std::thread::spawn(|| Self::proc(_status));

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

impl Drop for ThreadingBucketRateLimiter {
    fn drop(&mut self) {
        self.status
            .tx
            .lock()
            .unwrap()
            .send(())
            .expect("notifying thread panicked");

        if let Some(handle) = self.handle.lock().unwrap().take() {
            handle.join().expect("joining thread panicked");
        }
    }
}

impl ThreadingBucketRateLimiter {
    fn proc(status: std::sync::Arc<ThreadingBucketRateLimiterStatus>) {
        let mut instant = std::time::Instant::now();
        loop {
            instant += status.duration;
            match status
                .rx
                .lock()
                .unwrap()
                .recv_timeout(instant - std::time::Instant::now())
            {
                Ok(_) => break,
                Err(_) => {}
            }

            status
                .tokens
                .store(status.quota, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

/// A [RateLimiterService](crate::RateLimiterService) with [ThreadingBucketRateLimiter]
/// as its internal rate limiter implementation.
pub type ThreadingBucketRateLimiterService<S> =
    crate::RateLimiterService<S, ThreadingBucketRateLimiter>;

/// The [volo::Layer] implementation of [RateLimiterService](crate::RateLimiterService)
/// with [ThreadingBucketRateLimiter] as its internal rate limiter implementation.
pub type ThreadingBucketRateLimiterLayer = crate::RateLimiterLayer<ThreadingBucketRateLimiter>;
