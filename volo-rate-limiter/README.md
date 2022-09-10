This crate implements rate limiter layer for Volo-based service.

This crate usually used with crate `volo-limit-grpc` or `volo-limit-thrift`, depending on the type of service.

# Different Implementations of Rate Limiter

This crate provides multiple implementation options, see:
- [InaccurateBucketRateLimiter]
- [ThreadingBucketRateLimiter]
- [TokioBucketRateLimiter]

# Quick Start

Here using [ThreadingBucketRateLimiter] for examples.

## Volo gRPC Server

```rust
use volo_limit_grpc::GrpcLimitLayer;
use volo_rate_limiter::ThreadingBucketRateLimiterLayer;

#[volo::main]
async fn main() {
    let addr: SocketAddr = "[::]:8080".parse().unwrap();
    let addr = volo::net::Address::from(addr);

    volo_gen::ExampleSrver::new(S)
        // add the rate limiter layer as well as the gRPC limiter adaptor layer.
        .layer(GrpcLimitLayer(ThreadingBucketRateLimiterLayer::with_qps(100)))
        .run(addr)
        .await
        .unwrap();
}
```

## Volo Thrift Server

```rust
use volo_limit_thrift::ThriftLimitLayer;
use volo_rate_limiter::ThreadingBucketRateLimiterLayer;

#[volo::main]
async fn main() {
    let addr: SocketAddr = "[::]:8080".parse().unwrap();
    let addr = volo::net::Address::from(addr);

    volo_gen::ExampleServer::new(S)
        // add the concurrency limiter layer as well as the Thrift limiter adaptor layer.
        .layer(ThriftLimitLayer(ThreadingBucketRateLimiterLayer::with_qps(
            100,
        )))
        .run(addr)
        .await
        .unwrap();
}
```
