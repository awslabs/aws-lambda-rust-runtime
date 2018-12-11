# Rust Runtime for AWS Lambda

[![Build Status](https://travis-ci.org/awslabs/aws-lambda-rust-runtime.svg?branch=master)](https://travis-ci.org/awslabs/aws-lambda-rust-runtime)

This package makes it easy to run AWS Lambda Functions written in Rust. This workspace includes multiple crates:

* **`lambda-runtime-client`** is a client SDK for the Lambda Runtime APIs. You probably don't need to use this crate directly!
* **`lambda-runtime`** is a library that makes it easy to write Lambda functions in Rust.
* **`lambda-http`** is a library that makes it easy to write API Gateway proxy event focused Lambda functions in Rust.

## Example function

The code below creates a simple function that receives an event with a `greeting` and `name` field and returns a `GreetingResponse` message for the given name and greeting. Notice: to run these examples, we require a minimum Rust version of 1.31.

```rust,no_run
extern crate lambda_runtime as lambda;
extern crate serde_derive;
extern crate log;
extern crate simple_logger;

use serde_derive::{Serialize, Deserialize};
use lambda::{lambda, Context, error::HandlerError};
use log::error;
use std::error::Error;

#[derive(Serialize, Deserialize)]
struct GreetingEvent {
    greeting: String,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct GreetingResponse {
    message: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    lambda!(my_handler);

    Ok(())
}

fn my_handler(event: GreetingEvent, ctx: Context) -> Result<GreetingResponse, HandlerError> {
    if event.name == "" {
        error!("Empty name in request {}", ctx.aws_request_id);
        return Err(ctx.new_error("Empty name"));
    }

    Ok(GreetingResponse {
        message: format!("{}, {}!", event.greeting, event.name),
    })
}
```

The code above is the same as the [basic example](https://github.com/awslabs/aws-lambda-rust-runtime/tree/master/lambda-runtime/examples/basic.rs) in the `lambda-runtime` crate. To deploy the basic sample as a Lambda function, we first build it with `cargo`. Since Lambda uses Amazon Linux, you'll need to target your executable for an `x86_64-linux` platform.

```bash
$ cargo build -p lambda_runtime --example basic --release
```

For a custom runtime, AWS Lambda looks for an executable called `bootstrap` in the deployment package zip. Rename the generated `basic` executable to `bootstrap` and add it to a zip archive.

```bash
$ cp ./target/release/examples/basic ./bootstrap && zip lambda.zip bootstrap && rm bootstrap
```

Now that we have a deployment package (`lambda.zip`), we can use the [AWS CLI](https://aws.amazon.com/cli/) to create a new Lambda function. Make sure to replace the execution role with an existing role in your account!

```bash
$ aws lambda create-function --function-name rustTest \
  --handler doesnt.matter \
  --zip-file file://./lambda.zip \
  --runtime provided \
  --role arn:aws:iam::XXXXXXXXXXXXX:role/your_lambda_execution_role \
  --environment Variables={RUST_BACKTRACE=1} \
  --tracing-config Mode=Active
```

You can now test the function using the AWS CLI or the AWS Lambda console

```bash
$ aws lambda invoke --function-name rustTest \
  --payload '{"firstName": "world"}' \
  output.json
$ cat output.json  # Prints: {"message":"Hello, world!"}
```

## lambda-runtime-client

Defines the `RuntimeClient` trait and provides its `HttpRuntimeClient` implementation. The client fetches events and returns output as `Vec<u8>`.

For error reporting to the runtime APIs the library defines the `RuntimeApiError` trait and the `ErrorResponse` object. Custom errors for the APIs should implement the `to_response() -> ErrorResponse` method of the `RuntimeApiError` trait.

## lambda-runtime

This library makes it easy to create Rust executables for AWS lambda. The library defines a `lambda!()` macro. Call the `lambda!()` macro from your main method with a function that matches the `Handler` type:

```rust,ignore
pub type Handler<E, O> = fn(E, Context) -> Result<O, error::HandlerError>;
```

Optionally, you can pass your own instance of Tokio runtime to the `lambda!()` macro. See our [`with_custom_runtime.rs` example](https://github.com/awslabs/aws-lambda-rust-runtime/tree/master/lambda-runtime/examples/with_custom_runtime.rs)

## AWS event objects

This project does not currently include Lambda event struct defintions though we [intend to do so in the future](https://github.com/awslabs/aws-lambda-rust-runtime/issues/12). Instead, the community-maintained [aws_lambda_events](https://crates.io/crates/aws_lambda_events) crate can be leveraged to provide strongly-typed Lambda event structs. You can create your own custom event objects and their corresponding structs as well.

## Custom event objects

To serialize and deserialize events and responses, we suggest using the use the [`serde`](https://github.com/serde-rs/serde) library. To receive custom events, annotate your structure with Serde's macros:

```rust
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use serde_derive::{Serialize, Deserialize};
use serde_json::json;
use std::error::Error;

#[derive(Serialize, Deserialize)]
pub struct NewIceCreamEvent {
  pub flavors: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct NewIceCreamResponse {
  pub flavors_added_count: usize,
}

fn main() -> Result<(), Box<Error>> {
    let flavors = json!({
      "flavors": [
        "Nocciola",
        "抹茶",
        "आम"
      ]
    });

    let event: NewIceCreamEvent = serde_json::from_value(flavors)?;
    let response = NewIceCreamResponse {
        flavors_added_count: event.flavors.len(),
    };
    serde_json::to_string(&response)?;

    Ok(())
}
```
