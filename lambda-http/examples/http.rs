use std::error::Error;

use lambda_http::{lambda, IntoResponse, Request, RequestExt, Response};
use lambda_runtime::{error::HandlerError, Context};
use log::{self, error};
use simple_logger;

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Debug)?;
    lambda!(my_handler);

    Ok(())
}

fn my_handler(e: Request, c: Context) -> Result<impl IntoResponse, HandlerError> {
    Ok(match e.query_string_parameters().get("first_name") {
        Some(first_name) => format!("Hello, {}!", first_name).into_response(),
        _ => {
            error!("Empty first name in request {}", c.aws_request_id);
            Response::builder()
                .status(400)
                .body("Empty first name".into())
                .expect("failed to render response")
        }
    })
}
