# next (unreleased)

- **New**: Almost everything!

  Heavy restructuring was done in order to simply the runtime and to enable asynchronous handlers by default.

  The crate formerly known as `lambda_runtime` is now simply `lambda`, `lambda_http` remains the same.

  Both crates have note worth breaking api changes. The following is a rough approximation is what a transition to the new runtime will look like for your handlers

  → `Cargo.toml`

  ```diff
  [dependencies]
  - lambda_runtime = "0.2"
  + tokio = { version = "0.2", features = ["macros"] }
  + lambda = { git = "https://github.com/awslabs/aws-lambda-rust-runtime/", branch = "master"}
    ```
  
  > note: the runtime is `std::future::Future` based and does not rely   directly on tokio but tokio serves a very good default runtime for   driving `std::future::Future`s to completion
  
  → `main.rs`
  
  ```diff
  -use lambda_runtime::{error::HandlerError, lambda, Context};
  +use lambda::{handler_fn, Context};
  use serde_json::Value;
  
  +type Error = Box<dyn std::error::Error + Sync + Send + 'static>;

  -fn main() {
  -    lambda!(handler)
  
  +#[tokio::main]
  +async fn main() -> Result<(), Error> {
  +    lambda::run(handler_fn(handler)).await?;
  +    Ok(())
  }
  
  -fn handler(
  -    event: Value,
  -    _: Context,
  -) -> Result<Value, HandlerError> {
  +async fn handler(
  +    event: Value,
  +    _:Context
  +) -> Result<Value, Error> {
      Ok(event)
  }
  ```
  
  Please see the examples directories for more inspiration.
  
- **New:** the `lambda_http` crate now supports [API Gateway HTTP API](https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api.html) lambda triggers.

# 0.2.1

- **New**: The `lambda_http` crate now exposes mock helper methods for `RequestExt` under `cfg(test)` builds to facilitate straight forward unit testability of handlers.
- **New**: The `lambda_http` crate now exposes two new functions for deserializing requests from text and raw IO: `lambda_http::request::{from_str,from_reader}`.

# 0.2.0

- **New**: We created lambda_runtime_core crate that implements the runtime's main loop and supports handlers that accept and return Vec<u8>. ([#53](https://github.com/awslabs/aws-lambda-rust-runtime/issues/53))
- **New**: The primary lambda_runtime crate is a wrapper on top of the lambda_runtime_core handler ([#53](https://github.com/awslabs/aws-lambda-rust-runtime/issues/53))
- **New**: The lambda_http crate, which enables support for API Gateway or ALB requests, treating them as Request structs from the http crate ([#18 by @softprops](https://github.com/awslabs/aws-lambda-rust-runtime/issues/18)).
- **New**: The lambda_runtime_errors crate introduces the LambdaErrorExt trait that enables the ability to specify custom errorType values for the Lambda Runtime API. The change also includes a companion derive crate that makes it easy to automatically generate LambdaErrorExt implementations for crate-local error types ([#63](https://github.com/awslabs/aws-lambda-rust-runtime/issues/63)).
- **Fix**: Handlers can now return any error type ([#54](https://github.com/awslabs/aws-lambda-rust-runtime/issues/54))
- **Fix**: Support for closures as handlers ([#19 by @srijs](https://github.com/awslabs/aws-lambda-rust-runtime/issues/19)).
- **Fix**: Multiple bug fixes and performance improvements (thanks @Sh4rK).

# 0.1.0

- Initial Release
