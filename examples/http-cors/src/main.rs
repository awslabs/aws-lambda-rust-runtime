use lambda_http::{
    http::Method, service_fn, tower::ServiceBuilder, Body, Error, IntoResponse, Request, RequestExt, Response,
};
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // The runtime logging can be enabled here by initializing `tracing` with `tracing-subscriber`
    // While `tracing` is used internally, `log` can be used as well if preferred.
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    // Define a layer to inject CORS headers
    let cors_layer = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_origin(Any);

    let handler = ServiceBuilder::new()
        // Add the CORS layer to the service
        .layer(cors_layer)
        .service(service_fn(func));

    lambda_http::run(handler).await?;
    Ok(())
}

async fn func(event: Request) -> Result<Response<Body>, Error> {
    Ok(match event.query_string_parameters().first("first_name") {
        Some(first_name) => format!("Hello, {}!", first_name).into_response().await,
        _ => Response::builder()
            .status(400)
            .body("Empty first name".into())
            .expect("failed to render response"),
    })
}
