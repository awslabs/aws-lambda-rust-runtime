use lambda_http::{tower::ServiceBuilder, Body, Error, IntoResponse, Request, Response};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let service = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .service_fn(handler);
    lambda_http::run(service).await?;
    Ok(())
}

async fn handler(_event: Request) -> Result<Response<Body>, Error> {
    Ok("Success".into_response().await)
}
