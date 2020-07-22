use lambda::{handler_fn, Context};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use simple_logger;

/// A shorthand for `Box<dyn std::error::Error + Send + Sync + 'static>`
/// type required by aws-lambda-rust-runtime.
pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    // can be replaced with any other method of initializing `log`
    simple_logger::init_with_level(log::Level::Info)?;

    let func = handler_fn(my_handler);
    lambda::run(func).await?;
    Ok(())
}

pub(crate) async fn my_handler(event: Value, ctx: Context) -> Result<Value, Error> {
    // extract some useful info from the request
    let command = match serde_json::from_value::<Request>(event) {
        Err(e) => {
            return Err(Box::new(e));
        }
        Ok(v) => v.command,
    };

    // prepare the response
    let resp = Response {
        req_id: ctx.request_id,
        msg: format!("Command {} executed.", command),
    };

    // return `Response` as JSON
    Ok(json!(resp))
}

/// This is a made up example of what a response structure may look like.
/// There is no restriction on what it can be. The runtime requires responses
/// to be serialized into `serde_json::Value`. The runtime pays no attention
/// to the contents of the response payload.
#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String,
}

/// This is also a made up example. Requests come into the runtime as unicode
/// strings and are converted into `serde_json::Value`, which can map to your
/// own structures. The runtime pays no attention to the
/// contents of the request payload.
#[derive(Deserialize)]
struct Request {
    command: String,
}
