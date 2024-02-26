//! This example demonstrates how [axum middleware](https://docs.rs/axum/latest/axum/middleware/index.html)
//! can be implemented.
//!
//! To test this:
//! ```sh
//! # start the local server
//! cargo lambda watch
//! # Then send through an example request
//! cargo lambda invoke --data-example apigw-request
//! ```

use axum::{response::Json, routing::post, Router};
use lambda_http::request::RequestContext::ApiGatewayV1;
use lambda_http::{run, tracing, Error};
use serde_json::{json, Value};

// Sample middleware that logs the request id
async fn mw_sample(req: axum::extract::Request, next: axum::middleware::Next) -> impl axum::response::IntoResponse {
    let context = req.extensions().get::<lambda_http::request::RequestContext>();
    if let Some(ApiGatewayV1(ctx)) = context {
        tracing::info!("RequestId = {:?}", ctx.request_id);
    }
    next.run(req).await
}

async fn handler_sample(body: Json<Value>) -> Json<Value> {
    Json(json!({ "echo":  *body }))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    let app = Router::new()
        .route("/testStage/hello/world", post(handler_sample))
        .route_layer(axum::middleware::from_fn(mw_sample));

    run(app).await
}
