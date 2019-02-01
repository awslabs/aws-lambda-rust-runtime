use bytes::Bytes;
use simple_lambda_runtime::{lambda, Error};

fn main() {
    let handler = |event: Bytes| -> Result<Bytes, Error> { unimplemented!() };
    let catch = |err: Error| -> String { unimplemented!() };

    lambda!(handler, on_err = catch);
}
