use crate::RateLimitError;
use crate::RateLimiter;

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
    ) -> Result<Result<S::Response, S::Error>, RateLimitError>
    where
        's: 'cx,
    {
        match self.limiter.acquire() {
            Ok(_) => Ok(self.inner.call(cx, req).await),
            Err(_) => Err(RateLimitError),
        }
    }
}

/// The implementation of [volo::Layer] for [RateLimiterService].
pub struct RateLimiterLayer<L>(pub L);

impl<S, L> volo::Layer<S> for RateLimiterLayer<L>
where
    L: RateLimiter,
{
    type Service = RateLimiterService<S, L>;

    fn layer(self, inner: S) -> Self::Service {
        RateLimiterService {
            inner: inner,
            limiter: std::sync::Arc::new(self.0),
        }
    }
}
