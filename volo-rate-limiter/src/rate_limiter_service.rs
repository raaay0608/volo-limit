use crate::RateLimiter;
use crate::RateLimiterError;

/// A template type that wrappes a [RateLimiter] implmentation as a [volo::Service].
#[derive(Clone)]
pub struct RateLimiterService<S, L> {
    inner: S,
    limiter: std::sync::Arc<L>,
}

#[volo::service]
impl<Cx, Request, S, L> volo::Service<Cx, Request> for RateLimiterService<S, L>
where
    Request: Send + 'static,
    S: Send + 'static + volo::Service<Cx, Request, Response: Send>,
    Cx: Send + 'static,
    L: RateLimiter + std::marker::Sync,
{
    async fn call<'cx, 's>(
        &'s mut self,
        cx: &'cx mut Cx,
        req: Request,
    ) -> Result<Result<S::Response, S::Error>, RateLimiterError>
    where
        's: 'cx,
    {
        match self.limiter.acquire() {
            Ok(_) => Ok(self.inner.call(cx, req).await),
            Err(_) => Err(RateLimiterError),
        }
    }
}

/// The implementation of [volo::Layer] for [RateLimiterService].
pub struct RateLimiterLayer<L> {
    duration: std::time::Duration,
    quota: u64,

    limiter: std::marker::PhantomData<L>,
}

impl<L> RateLimiterLayer<L> {
    /// Creates a new [RateLimiterLayer] with limit duration and quota.
    pub fn new(duration: impl Into<std::time::Duration>, quota: u64) -> Self {
        Self {
            duration: duration.into(),
            quota,
            limiter: std::marker::PhantomData,
        }
    }

    /// Creates a new [RateLimiterLayer] with a limit duration of a second.
    pub fn with_qps(qps: u64) -> Self {
        Self {
            duration: std::time::Duration::from_secs(1),
            quota: qps,
            limiter: std::marker::PhantomData,
        }
    }
}

impl<S, L> volo::Layer<S> for RateLimiterLayer<L>
where
    L: RateLimiter,
{
    type Service = RateLimiterService<S, L>;

    fn layer(self, inner: S) -> Self::Service {
        RateLimiterService {
            inner: inner,
            limiter: std::sync::Arc::new(L::new(self.duration, self.quota)),
        }
    }
}
