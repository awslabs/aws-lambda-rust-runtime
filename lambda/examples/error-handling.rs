/// See https://github.com/awslabs/aws-lambda-rust-runtime for more info on Rust runtime for AWS Lambda
use lambda::handler_fn;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::File;

/// A shorthand for `Box<dyn std::error::Error + Send + Sync + 'static>` type required by aws-lambda-rust-runtime.
pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

/// A simple Lambda request structure with just one field
/// that tells the Lambda what is expected of it.
#[derive(Deserialize)]
struct Request {
    event_type: EventType,
}

/// Event types that tell our Lambda what to do do.
#[derive(Deserialize, PartialEq)]
enum EventType {
    Response,
    ExternalError,
    SimpleError,
    CustomError,
    Panic,
}

/// A simple Lambda response structure.
#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String,
}

#[derive(Debug, Serialize)]
struct CustomError {
    is_authenticated: bool,
    req_id: String,
    msg: String,
}

impl std::error::Error for CustomError {
    // this implementation required `Debug` and `Display` traits
}

impl std::fmt::Display for CustomError {
    /// Display the error struct as a JSON string
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_as_json = json!(self).to_string();
        write!(f, "{}", err_as_json)
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // The runtime logging can be enabled here by initializing `log` with `simple_logger`
    // or another compatible crate. The runtime is using `tracing` internally.
    // You can comment out the `simple_logger` init line and uncomment the following block to
    // use `tracing` in the handler function.
    //
    simple_logger::init_with_level(log::Level::Info)?;
    /*
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // this needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();
    */

    // call the actual handler of the request
    let func = handler_fn(func);
    lambda::run(func).await?;
    Ok(())
}

/// The actual handler of the Lambda request.
pub(crate) async fn func(event: Value, ctx: lambda::Context) -> Result<Value, Error> {
    // check what action was requested
    match serde_json::from_value::<Request>(event)?.event_type {
        EventType::SimpleError => {
            // generate a simple text message error using `simple_error` crate
            return Err(Box::new(simple_error::SimpleError::new("A simple error as requested!")));
        }
        EventType::CustomError => {
            // generate a custom error using our own structure
            let cust_err = CustomError {
                is_authenticated: ctx.identity.is_some(),
                req_id: ctx.request_id,
                msg: "A custom error as requested!".into(),
            };
            return Err(Box::new(cust_err));
        }
        EventType::ExternalError => {
            // try to open a non-existent file to get an error and propagate it with `?`
            let _file = File::open("non-existent-file.txt")?;

            // it should never execute past the above line
            unreachable!();
        }
        EventType::Panic => {
            panic!();
        }
        EventType::Response => {
            // generate and return an OK response in JSON format
            let resp = Response {
                req_id: ctx.request_id,
                msg: "OK".into(),
            };

            return Ok(json!(resp));
        }
    }
}
