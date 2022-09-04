#![feature(associated_type_bounds)]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

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
