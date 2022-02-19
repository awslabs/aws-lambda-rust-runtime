use http::Method;
use lambda_http::{service_fn, tower::ServiceBuilder, Body, Error, IntoResponse, Request, RequestExt, Response};
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() -> Result<(), Error> {
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
        Some(first_name) => format!("Hello, {}!", first_name).into_response(),
        _ => Response::builder()
            .status(400)
            .body("Empty first name".into())
            .expect("failed to render response"),
    })
}
