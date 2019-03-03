# next (unreleased)

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
