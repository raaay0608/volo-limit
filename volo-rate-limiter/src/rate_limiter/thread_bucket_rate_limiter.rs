/// `ThreadBucketRateLimiter` is a bucket rate limiter implementation,
/// using a dedicated thread as token producer.
#[derive(Clone)]
pub struct ThreadBucketRateLimiter {
    status: std::sync::Arc<ThreadBucketRateLimiterStatus>,

    // wrapped in `Arc<Mutex<...>>` to satisfy `Clone` and `Send` requirements.
    handle: std::sync::Arc<std::sync::Mutex<Option<std::thread::JoinHandle<()>>>>,
}

struct ThreadBucketRateLimiterStatus {
    duration: std::time::Duration,
    quota: i64,

    tokens: std::sync::atomic::AtomicI64,

    // `tx` and `rx` are used as thread termination notifier.
    tx: std::sync::Mutex<std::sync::mpsc::Sender<()>>,
    rx: std::sync::Mutex<std::sync::mpsc::Receiver<()>>,
}

impl crate::RateLimiter for ThreadBucketRateLimiter {
    fn new(duration: impl Into<std::time::Duration>, quota: u64) -> Self {
        let quota: i64 = quota.try_into().expect("limit quota out of range");

        let (tx, rx) = std::sync::mpsc::channel();

        let status = std::sync::Arc::new(ThreadBucketRateLimiterStatus {
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

impl Drop for ThreadBucketRateLimiter {
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

impl ThreadBucketRateLimiter {
    fn proc(status: std::sync::Arc<ThreadBucketRateLimiterStatus>) {
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

/// `RateLimiterService` with `ThreadBucketRateLimiter` as its internal limiter implementation.
pub type ThreadBucketRateLimiterService<S> = crate::RateLimiterService<S, ThreadBucketRateLimiter>;

/// The `volo::layer` implementation of `RateLimiterService` with `ThreadBucketRateLimiter` as its internal limiter implementation.
pub type ThreadBucketRateLimiterLayer = crate::RateLimiterLayer<ThreadBucketRateLimiter>;
