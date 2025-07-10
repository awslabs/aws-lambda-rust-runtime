use lambda_http::{
    run,
    tower::ServiceBuilder,
    tracing::{self, Level},
    Error, Request, Response,
};
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};

async fn handler(_req: Request) -> Result<Response<String>, Error> {
    Ok(Response::new("Success".into()))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    let layer = TraceLayer::new_for_http()
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO));

    let service = ServiceBuilder::new().layer(layer).service_fn(handler);

    run(service).await?;
    Ok(())
}
