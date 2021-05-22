// This example demonstrates use of shared resources such as DB connections
// or local caches that can be initialized at the start of the runtime and
// reused by subsequent lambda handler calls.
// Run it with the following input:
// { "command": "do something" }

use lambda_runtime::{handler_fn, Context, Error};
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;

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

struct SharedClient {
    name: &'static str,
}

impl SharedClient {
    fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn response(&self, req_id: String, command: String) -> Response {
        Response {
            req_id,
            msg: format!("Command {} executed by {}.", command, self.name),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    // can be replaced with any other method of initializing `log`
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();

    let client = SharedClient::new("Shared Client 1 (perhaps a database)");
    let client_ref = &client;
    lambda_runtime::run(handler_fn(move |event: Request, ctx: Context| async move {
        let command = event.command;
        Ok::<Response, Error>(client_ref.response(ctx.request_id, command))
    }))
    .await?;
    Ok(())
}
