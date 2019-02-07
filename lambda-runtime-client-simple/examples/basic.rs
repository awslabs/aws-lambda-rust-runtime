use bytes::Bytes;
use futures::Future;
use simple_lambda_runtime::{lambda, Error};

fn main() {
    lambda!(handler);
}

fn handler(event: Bytes) -> Box<Future<Item = Bytes, Error = Error>> {
    unimplemented!()
}
