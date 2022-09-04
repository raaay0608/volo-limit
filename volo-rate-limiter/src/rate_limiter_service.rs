use crate::RateLimiter;
use crate::RateLimiterError;

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

pub struct RateLimiterLayer<L> {
    interval: std::time::Duration,
    limit: u64,

    limiter: std::marker::PhantomData<L>,
}

impl<L> RateLimiterLayer<L> {
    pub fn new(interval: impl Into<std::time::Duration>, limit: u64) -> Self {
        Self {
            interval: interval.into(),
            limit,
            limiter: std::marker::PhantomData,
        }
    }

    pub fn with_qps(qps: u64) -> Self {
        Self {
            interval: std::time::Duration::from_secs(1),
            limit: qps,
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
            limiter: std::sync::Arc::new(L::new(self.interval, self.limit)),
        }
    }
}
