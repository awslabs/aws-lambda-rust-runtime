// This example requires the following input to succeed:
// { "command": "do something" }

use lambda_runtime::{handler_fn, Context, Error};
use serde::{Deserialize, Serialize};

/// This is also a made-up example. Requests come into the runtime as unicode
/// strings in json format, which can map to any structure that implements `serde::Deserialize`
/// The runtime pays no attention to the contents of the request payload.
#[derive(Deserialize)]
struct Request {
    command: String,
}

/// This is a made-up example of what a response structure may look like.
/// There is no restriction on what it can be. The runtime requires responses
/// to be serialized into json. The runtime pays no attention
/// to the contents of the response payload.
#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // this needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let func = handler_fn(my_handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

pub(crate) async fn my_handler(event: Request, ctx: Context) -> Result<Response, Error> {
    // extract some useful info from the request
    let command = event.command;

    // prepare the response
    let resp = Response {
        req_id: ctx.request_id,
        msg: format!("Command {} executed.", command),
    };

    // return `Response` (it will be serialized to JSON automatically by the runtime)
    Ok(resp)
}
