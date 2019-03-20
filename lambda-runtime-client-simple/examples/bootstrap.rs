#[macro_use]
extern crate tokio_trace;

use bytes::Bytes;
use futures::{future::ok, Future};
use hyper::Body;
use simple_lambda_runtime::{lambda, Error};

fn main() -> Result<(), Error> {
    lambda!(handler)
}

fn handler(event: Body) -> Box<dyn Future<Item = Bytes, Error = Error> + Send> {
    info!("Received event");
    Box::new(ok(Bytes::from("hello")))
}
