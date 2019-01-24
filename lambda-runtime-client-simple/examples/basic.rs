use simple_lambda_runtime::{start, RuntimeError, lambda};
use std::error::Error;
use bytes::Bytes;

fn main() {
    let handler = |event: Bytes| -> Result<Bytes, RuntimeError> {
        unimplemented!()
    };
    let catch = |err: RuntimeError| -> String {
        unimplemented!()
    };

    lambda!(handler, on_err = catch);
}