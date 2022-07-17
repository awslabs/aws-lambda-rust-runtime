use lambda_http::{run, tower::ServiceBuilder, Error};
use lambda_http::{Request, Response};
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

async fn handler(_req: Request) -> Result<Response<String>, Error> {
    Ok(Response::new("Success".into()))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().without_time().init();

    let layer = TraceLayer::new_for_http()
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO));

    let service = ServiceBuilder::new().layer(layer).service_fn(handler);

    run(service).await?;
    Ok(())
}
