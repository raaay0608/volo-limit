#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

/// `ConcurrencyLimiterService` implements a basic concurrency limiter.
///
/// # Limitation
///
/// This limiter may not effects if the inner service does not perform any async operations.
/// This can happen with some pure compute services and caching services.
///
/// The reason is that, without any async operations, the service process becomes "atomic" that each worker will never begin to handle a new request until the current request is finished.
/// Base on this situation, the possibly maximum concurrency is the number of the workers (which usually equals to the number of CPU logical cores), and it may never reach the passed-in limitation.
#[derive(Clone)]
pub struct ConcurrencyLimiterService<S> {
    inner: S,
    status: std::sync::Arc<ConcurrencyLimiterServiceSharedStatus>,
}

struct ConcurrencyLimiterServiceSharedStatus {
    limit: u64,
    curr: std::sync::atomic::AtomicU64,
}

/// `ConcurrencyLimiterServiceror` is the error type raised by `ConcurrencyLimiterService` on determining the requested will be rejected.
#[derive(Debug)]
pub struct ConcurrencyLimiterServiceror;

impl std::fmt::Display for ConcurrencyLimiterServiceror {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "concurrency limited")
    }
}

impl std::error::Error for ConcurrencyLimiterServiceror {}

#[volo::service]
impl<Cx, Request, S> volo::Service<Cx, Request> for ConcurrencyLimiterService<S>
where
    Request: Send + 'static,
    S: Send + 'static + volo::Service<Cx, Request>,
    Cx: Send + 'static,
{
    async fn call<'cx, 's>(
        &'s mut self,
        cx: &'cx mut Cx,
        req: Request,
    ) -> Result<Result<S::Response, S::Error>, ConcurrencyLimiterServiceror>
    where
        's: 'cx,
    {
        let curr = self
            .status
            .curr
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if curr > self.status.limit {
            self.status
                .curr
                .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            return Err(ConcurrencyLimiterServiceror);
        }

        let res = self.inner.call(cx, req).await;

        self.status
            .curr
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);

        Ok(res)
    }
}

pub struct ConcurrencyLimiterServiceLayer {
    limit: u64,
}

impl ConcurrencyLimiterServiceLayer {
    pub fn with_concurrency_limit(limit: u64) -> Self {
        Self { limit }
    }

    pub fn with_limit(self, limit: u64) -> Self {
        Self { limit }
    }
}

impl<S> volo::Layer<S> for ConcurrencyLimiterServiceLayer {
    type Service = ConcurrencyLimiterService<S>;

    fn layer(self, inner: S) -> Self::Service {
        ConcurrencyLimiterService {
            inner,
            status: std::sync::Arc::new(ConcurrencyLimiterServiceSharedStatus {
                limit: self.limit,
                curr: std::sync::atomic::AtomicU64::new(0),
            }),
        }
    }
}
