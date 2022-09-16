#![doc = include_str!("../README.md")]
#![feature(associated_type_bounds)]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

/// An adaptor layer between a limiter and a gRPC service.
///
/// Given an arbitary [volo::Service] instance called `S`, a type-independent limiter service is a
/// [volo::Service] with its return type is `Result<Result<S::Response, S::Error>, LimitError>`, where
/// `LimitError` is the error type defined by the limiter and will be returned if request is
/// determined to be limited.
///
/// The `GrpcLimitService` accepts the `Result<Result<S::Response, S::Error>, LimitError>` from limiter
/// and returns `Result<S::Response, S::Error>`, by converting the `Err(LimitError)` into a gRPC calling
/// error with a RESOURCE_EXHAUSTED status.
#[derive(Clone)]
pub struct GrpcLimitService<S>(S);

#[volo::service]
impl<Cx, Request, S, OriginResponse, LimitError> volo::Service<Cx, Request> for GrpcLimitService<S>
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
            Response = Result<OriginResponse, volo_grpc::Status>,
            Error = LimitError,
        >,
{
    async fn call(
        &mut self,
        cx: &mut Cx,
        req: Request,
    ) -> Result<OriginResponse, volo_grpc::Status> {
        match self.0.call(cx, req).await {
            Ok(res) => res,
            Err(e) => Err(volo_grpc::Status::resource_exhausted(e.to_string())),
        }
    }
}

///The [volo::Layer] implementation for [GrpcLimitService].
pub struct GrpcLimitLayer<L>(pub L);

impl<S, L, S0> volo::Layer<S> for GrpcLimitLayer<L>
where
    L: volo::Layer<S, Service = S0>,
{
    type Service = GrpcLimitService<L::Service>;

    fn layer(self, inner: S) -> Self::Service {
        GrpcLimitService(self.0.layer(inner))
    }
}
