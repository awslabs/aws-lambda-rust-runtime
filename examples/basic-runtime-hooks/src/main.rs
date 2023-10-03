// This example demonstrates use of shared resources such as DB connections
// or local caches that can be initialized at the start of the runtime and
// reused by subsequent lambda handler calls.
// Run it with the following input:
// { "command": "do something" }

use lambda_runtime::{crac::Resource, service_fn, Error, LambdaEvent, Runtime};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use uuid::Uuid;

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
    secret: String,
}

struct SharedClient {
    name: &'static str,
    secret: RefCell<String>,
}

impl SharedClient {
    fn new(name: &'static str, secret: String) -> Self {
        Self {
            name,
            secret: RefCell::new(secret),
        }
    }

    fn response(&self, req_id: String, command: String) -> Response {
        Response {
            req_id,
            msg: format!("Command {} executed by {}.", command, self.name),
            secret: self.secret.borrow().clone(),
        }
    }
}

impl Resource for SharedClient {
    fn before_checkpoint(&self) -> Result<(), Error> {
        // clear the secret before checkpointing
        *self.secret.borrow_mut() = String::new();
        tracing::info!("in before_checkpoint: secret={:?}", self.secret.borrow());
        Ok(())
    }
    fn after_restore(&self) -> Result<(), Error> {
        // regenerate the secret after restoring
        let secret = Uuid::new_v4().to_string();
        *self.secret.borrow_mut() = secret;
        tracing::info!("in after_restore: secret={:?}", self.secret.borrow());
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let secret = Uuid::new_v4().to_string();
    let client = SharedClient::new("Shared Client 1 (perhaps a database)", secret);
    let client_ref = &client;
    tracing::info!("In main function: secret={:?}", client_ref.secret.borrow());

    Runtime::new()
        .register(client_ref)
        .run(service_fn(move |event: LambdaEvent<Request>| async move {
            tracing::info!("In handler function: secret={:?}", client_ref.secret.borrow());
            let command = event.payload.command;
            Ok::<Response, Error>(client_ref.response(event.context.request_id, command))
        }))
        .await?;
    Ok(())
}
