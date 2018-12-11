extern crate lambda_http as http;
extern crate lambda_runtime as runtime;
extern crate log;
extern crate simple_logger;

use http::{lambda, IntoResponse, Request, RequestExt, Response};
use runtime::{error::HandlerError, Context};

use log::error;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
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
