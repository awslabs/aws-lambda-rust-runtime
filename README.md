# Rust runtime for AWS Lambda

This package makes it easy to run AWS Lambda Functions written in Rust. This workspace includes multiple crates:
* **`lambda-runtime-client`** is a client SDK for the Lambda runtime APIs
* **`lambda-runtime`** is a library that makes it easy to write Lambda functions in rust as executables

## Example function
The code below creates a simple function that receives an event with a `firstName` property and returns a hello world message for the given first name.

```rust
#[macro_use]
extern crate lambda_runtime as lambda;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate simple_logger;

use lambda::error::HandlerError;

use std::error::Error;

#[derive(Deserialize, Clone)]
struct CustomEvent {
    #[serde(rename = "firstName")]
    first_name: String,
}

#[derive(Serialize, Clone)]
struct CustomOutput {
    message: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Info)?;
    lambda!(my_handler);

    Ok(())
}

fn my_handler(e: CustomEvent, c: lambda::Context) -> Result<CustomOutput, HandlerError> {
    if e.first_name == "" {
        error!("Empty first name in request {}", c.aws_request_id);
        return Err(c.new_error("Empty first name"));
    }

    Ok(CustomOutput {
        message: format!("Hello, {}!", e.first_name),
    })
}
```

The code above is the same as the [basic example](https://github.com/awslabs/aws-lambda-rust-runtime/tree/master/lambda-runtime/examples/basic.rs) in the `lambda-runtime` crate. To deploy the basic sample as a Lambda function, we first build it with `cargo`. Remember that AWS Lambda uses Amazon Linux so you need to target your executable for an `x86_64-linux` platform.

```bash
$ cargo build -p lambda_runtime --example basic --release
```

For a custom runtime, AWS Lambda looks for an executable called `boostrap` in the deployment package zip. Rename the generated `basic` executable to `bootstrap` and add it to a zip archive.

```bash
$ cp ./target/release/examples/basic ./bootstrap && zip rust.zip bootstrap && rm bootstrap
```

Now that we have a deployment package (`rust.zip`), we can use the [AWS CLI](https://aws.amazon.com/cli/) to create a new Lambda function. Make sure to replace the execution role with an existing role in your account.

```bash
$ aws lambda create-function --function-name rustTest \
  --handler doesnt.matter \
  --zip-file file://./rust.zip \
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
```rust
pub type Handler<E, O> = fn(E, Context) -> Result<O, error::HandlerError>;
```

Optionally, you can pass your own instance of Tokio runtime to the `lambda!()` macro. See our [`with_custom_runtime.rs` example](https://github.com/awslabs/aws-lambda-rust-runtime/tree/master/lambda-runtime/examples/with_custom_runtime.rs)

## Custom event objects

To serialize and de-serialize events and responses we use the [`serde_json`](https://crates.io/crates/serde_json) library. To receive custom events, simply annotate your structure with Serde's macros:

```rust
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[derive(Deserialize)]
pub struct MyEvent {
  pub records: Vec<String>,
}

#[derive(Serialize)]
pub struct MyEventResponse {
  pub message: String,
}
```
