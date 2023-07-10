use lambda_http::{
    http::Method, service_fn, tower::ServiceBuilder, Body, Error, IntoResponse, Request, RequestExt, Response,
};
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
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
    Ok(
        match event
            .query_string_parameters_ref()
            .and_then(|params| params.first("first_name"))
        {
            Some(first_name) => format!("Hello, {}!", first_name).into_response().await,
            None => Response::builder()
                .status(400)
                .body("Empty first name".into())
                .expect("failed to render response"),
        },
    )
}
