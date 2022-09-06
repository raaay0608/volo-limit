#![feature(associated_type_bounds)]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

/// `ThriftLimitService` is a adaptor layer between a limiter and a Thrift service.
///
/// Given a arbitary volo service called `S`, a protocol-independent limiter services is a
/// volo service with its return type is `Result<Result<S::Response, S::Error>, LimitError>`, where
/// `LimitError` is the error type defined by the limiter and will be returned if request is
/// determined to be limited.
///
/// The `ThriftLimitService` accepts the `Result<Result<S::Response, S::Error>, LimitError>` from limiter
/// and returns `Result<S::Response, S::Error>`, by converting the `Err(LimitError)` into a Thrift error
/// with "application error" as its error type, and "unknown" as its error kind.
#[derive(Clone)]
pub struct ThriftLimitService<S>(S);

#[volo::service]
impl<Cx, Request, S, OriginResponse, LimitError> volo::Service<Cx, Request>
    for ThriftLimitService<S>
where
    Cx: Send + 'static,
    Request: Send + 'static,
    OriginResponse: Send + 'static,
    LimitError: std::error::Error + Send + 'static,
    S: Send
        + 'static
        + volo::Service<
            Cx,
            Request,
            Response = Result<OriginResponse, pilota::AnyhowError>,
            Error = LimitError,
        >,
{
    async fn call(
        &mut self,
        cx: &mut Cx,
        req: Request,
    ) -> Result<OriginResponse, pilota::AnyhowError> {
        match self.0.call(cx, req).await {
            Ok(res) => res,
            Err(e) => Err(pilota::thrift::new_application_error(
                // XXX: Does thrift protocol have a proper error kind for limiting ?
                ::pilota::thrift::ApplicationErrorKind::Unknown,
                e.to_string(),
            )
            .into()),
        }
    }
}

/// `ThriftLimitLayer` is the `volo::layer` implementation for `ThriftLimitService`.
pub struct ThriftLimitLayer<L>(pub L);

impl<S, L, S0> volo::Layer<S> for ThriftLimitLayer<L>
where
    L: volo::Layer<S, Service = S0>,
{
    type Service = ThriftLimitService<L::Service>;

    fn layer(self, inner: S) -> Self::Service {
        ThriftLimitService(self.0.layer(inner))
    }
}
