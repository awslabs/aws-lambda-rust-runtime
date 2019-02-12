use bytes::Bytes;
use futures::Future;
use hyper::Body;
use simple_lambda_runtime::{lambda, Error};

fn main() -> Result<(), Error> {
    lambda!(handler)
}

fn handler(event: Body) -> Box<dyn Future<Item = Bytes, Error = Error> + Send> {
    unimplemented!()
}
