This crate implements a concurrency limiter layer for Volo-based service.

This crate usually used with crate `volo-limit-grpc` or `volo-limit-thrift`, depending on the type of service.

# Notice

This limiter in this crate may not limit if the service calling process does not contain any `async` operations.
This may happen with some of pure-computing servers and caching servers.

The reason here is that, without any async operations, the handling process of a requests (service's [call](volo::Service::call) method) becomes "atomic" that each worker will not begin to handle a new request until the current request is finished. Base on this situation, the possibly maximum concurrency is the number of the workers (usually equals to the number of logical CPU cores), which may never reach the passed-in limitation.


# Quick Start

## Volo gRPC Server

```rust
use volo_concurrency_limiter::ConcurrencyLimiterServiceLayer;
use volo_limit_grpc::GrpcLimitLayer;

#[volo::main]
async fn main() {
    let addr: SocketAddr = "[::]:8080".parse().unwrap();
    let addr = volo::net::Address::from(addr);

    volo_gen::ExampleSrver::new(S)
        // add the concurrency limiter layer as well as the gRPC limiter adaptor layer.
        .layer(GrpcLimitLayer(
            ConcurrencyLimiterServiceLayer::with_concurrency_limit(100),
        ))
        .run(addr)
        .await
        .unwrap();
}
```

## Volo Thrift Server

```rust
use volo_concurrency_limiter::ConcurrencyLimiterServiceLayer;
use volo_limit_thrift::ThriftLimitLayer;

#[volo::main]
async fn main() {
    let addr: SocketAddr = "[::]:8080".parse().unwrap();
    let addr = volo::net::Address::from(addr);

    volo_gen::ExampleServer::new(S)
        // add the concurrency limiter layer as well as the Thrift limiter adaptor layer.
        .layer(ThriftLimitLayer(
            ConcurrencyLimiterServiceLayer::with_concurrency_limit(100),
        ))
        .run(addr)
        .await
        .unwrap();
}
```
