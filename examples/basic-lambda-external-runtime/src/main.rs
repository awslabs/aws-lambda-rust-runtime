use std::{io, thread};

use futures_lite::future;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use tokio::runtime::Builder;

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

fn main() -> Result<(), io::Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    // Create a channel used to send and receive outputs from our lambda handler. Realistically, this would be either an unbounded channel
    // or a bounded channel with a higher capacity as needed.
    let (lambda_tx, lambda_rx) = async_channel::bounded(1);

    // Create a bounded channel used to communicate our shutdown signal across threads.
    let (shutdown_tx, shutdown_rx) = async_channel::bounded(1);

    // Build a single-threaded (or multi-threaded using Builder::new_multi_thread) runtime to spawn our lambda work onto.
    let tokio_runtime = Builder::new_current_thread()
        .thread_name("lambda-runtime")
        .enable_all()
        .build()
        .expect("build lambda runtime");

    // Run the lambda runtime worker thread to completion. The response is sent to the other "runtime" to be processed as needed.
    thread::spawn(move || {
        let func = service_fn(my_handler);
        if let Ok(response) = tokio_runtime.block_on(lambda_runtime::run(func)) {
            lambda_tx.send_blocking(response).expect("send lambda result");
        };
    });

    // Run the mock runtime to completion.
    my_runtime(move || future::block_on(app_runtime_task(lambda_rx.clone(), shutdown_tx.clone())));

    // Block the main thread until a shutdown signal is received.
    future::block_on(shutdown_rx.recv()).map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{:?}", err)))
}

pub(crate) async fn my_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // extract some useful info from the request
    let command = event.payload.command;

    // prepare the response
    let resp = Response {
        req_id: event.context.request_id,
        msg: format!("Command {} executed.", command),
    };

    // return `Response` (it will be serialized to JSON automatically by the runtime)
    Ok(resp)
}

/// A task to be ran on the custom runtime. Once a response from the lambda runtime is received then a shutdown signal
/// is sent to the main thread notifying the process to exit.
pub(crate) async fn app_runtime_task(lambda_rx: async_channel::Receiver<()>, shutdown_tx: async_channel::Sender<()>) {
    loop {
        // Receive the response sent by the lambda handle and process as needed.
        if let Ok(result) = lambda_rx.recv().await {
            tracing::debug!(?result);
            // We're ready to shutdown our app. Send the shutdown signal notifying the main thread to exit the process.
            shutdown_tx.send(()).await.expect("send shutdown signal");
            break;
        }

        // more app logic would be here...
    }
}

/// Construct the mock runtime worker thread(s) to spawn some work onto.
fn my_runtime(func: impl Fn() + Send + 'static) {
    thread::Builder::new()
        .name("my-runtime".into())
        .spawn(func)
        .expect("spawn my_runtime worker");
}
