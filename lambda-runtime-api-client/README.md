# AWS Lambda Runtime API Client

[![Docs](https://docs.rs/lambda_runtime_api_client/badge.svg)](https://docs.rs/lambda_runtime_api_client)

**`lambda-runtime-api-client`** is a library to interact with the AWS Lambda Runtime API.

This crate provides simple building blocks to send REST request to this API. You probably don't need to use this crate directly, look at [lambda_runtime](https://docs.rs/lambda_runtime) and [lambda_extension](https://docs.rs/lambda_extension) instead.

## Example

```rust,no_run
use http::{Method, Request};
use hyper::Body;
use lambda_runtime_api_client::{build_request, Client, Error};

fn register_request(extension_name: &str, events: &[&str]) -> Result<Request<Body>, Error> {
    let events = serde_json::json!({ "events": events });

    let req = build_request()
        .method(Method::POST)
        .uri("/2020-01-01/extension/register")
        .header("Lambda-Extension-Name", extension_name)
        .body(Body::from(serde_json::to_string(&events)?))?;

    Ok(req)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::builder().build()?;
    let request = register_request("my_extension", &["INVOKE"])?;

    client.call(request).await
}
```
