extern crate lambda_runtime as lambda;
extern crate lambda_http;
extern crate log;
extern crate simple_logger;

use lambda_http::{lambda, Request, Response};
use lambda::{error::HandlerError, Context};

use log::error;
use serde_derive::{Deserialize, Serialize};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    lambda!(my_handler);

    Ok(())
}

fn my_handler(e: Request, c: Context) -> Result<Response, HandlerError> {
    let name = e.query_string_parameters().get("first_name");
    Ok(match name {
      Some(first_name) => {
        Response::new(format!("Hello, {}!", first_name).into())
      },
      _ => {
        error!("Empty first name in request {}", c.aws_request_id);
        Response::builder(400).body("Empty first name").expect("failed to render response").into()
      }
    })
}
