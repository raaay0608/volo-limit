#![doc = include_str!("../README.md")]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

/// The implementation of basic concurrency limiter as [volo::Service].
///
/// For the informations and notices, see the [documentation page of this crate](crate).
#[derive(Clone)]
pub struct ConcurrencyLimiterService<S> {
    inner: S,
    status: std::sync::Arc<ConcurrencyLimiterServiceSharedStatus>,
}

struct ConcurrencyLimiterServiceSharedStatus {
    limit: u64,
    curr: std::sync::atomic::AtomicU64,
}

/// The error type returned by [ConcurrencyLimiterService] when determining that the request will be rejected.
#[derive(Debug)]
pub struct ConcurrencyLimitError;

impl std::fmt::Display for ConcurrencyLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "concurrency limited")
    }
}

impl std::error::Error for ConcurrencyLimitError {}

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
    ) -> Result<Result<S::Response, S::Error>, ConcurrencyLimitError>
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
            return Err(ConcurrencyLimitError);
        }

        let res = self.inner.call(cx, req).await;

        self.status
            .curr
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);

        Ok(res)
    }
}

/// The [volo::Layer] implementation for [ConcurrencyLimiterService].
pub struct ConcurrencyLimiterServiceLayer {
    limit: u64,
}

impl ConcurrencyLimiterServiceLayer {
    pub fn with_concurrency_limit(limit: u64) -> Self {
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
