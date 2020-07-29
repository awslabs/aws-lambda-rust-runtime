# Rust Runtime for AWS Lambda

This package makes it easy to run AWS Lambda Functions written in Rust. This workspace includes multiple crates:

- [![Docs](https://docs.rs/lambda/badge.svg)](https://docs.rs/lambda) **`lambda`** is a library that provides a Lambda runtime for applications written in Rust.
- [![Docs](https://docs.rs/lambda_http/badge.svg)](https://docs.rs/lambda_http) **`lambda-http`** is a library that makes it easy to write API Gateway proxy event focused Lambda functions in Rust.

## Example function

The code below creates a simple function that receives an event with a `firstName` field and returns a message to the caller. Notice: this crate is tested against latest stable Rust.

```rust,no_run
use lambda::{handler_fn, Context};
use serde_json::{json, Value};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = handler_fn(func);
    lambda::run(func).await?;
    Ok(())
}

async fn func(event: Value, _: Context) -> Result<Value, Error> {
    let first_name = event["firstName"].as_str().unwrap_or("world");

    Ok(json!({ "message": format!("Hello, {}!", first_name) }))
}
```

The code above is the same as the [basic example](https://github.com/awslabs/aws-lambda-rust-runtime/blob/master/lambda/examples/hello-without-macro.rs) in the `lambda` crate.

### Deployment

There are currently multiple ways of building this package: manually with the AWS CLI, and with the [Serverless framework](https://serverless.com/framework/).

#### AWS CLI

To deploy the basic sample as a Lambda function using the [AWS CLI](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-welcome.html), we first need to manually build it with [`cargo`](https://doc.rust-lang.org/cargo/). Since Lambda uses Amazon Linux, you'll need to target your executable for an `x86_64-unknown-linux-musl` platform.

Run this script once to add the new target:
```bash
$ rustup target add x86_64-unknown-linux-musl
```

Compile one of the examples as a _release_ with a specific _target_ for deployment to AWS:
```bash
$ cargo build -p lambda --example hello --release --target x86_64-unknown-linux-musl
```

For [a custom runtime](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-custom.html), AWS Lambda looks for an executable called `bootstrap` in the deployment package zip. Rename the generated `basic` executable to `bootstrap` and add it to a zip archive.

```bash
$ cp ./target/release/examples/hello ./bootstrap && zip lambda.zip bootstrap && rm bootstrap
```

Now that we have a deployment package (`lambda.zip`), we can use the [AWS CLI](https://aws.amazon.com/cli/) to create a new Lambda function. Make sure to replace the execution role with an existing role in your account!

```bash
$ aws lambda create-function --function-name rustTest \
  --handler doesnt.matter \
  --zip-file fileb://./lambda.zip \
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
$ cat output.json  # Prints: {"message": "Hello, world!"}
```

**Note:** `--cli-binary-format raw-in-base64-out` is a required
  argument when using the AWS CLI version 2. [More Information](https://docs.aws.amazon.com/cli/latest/userguide/cliv2-migration.html#cliv2-migration-binaryparam)

#### Serverless Framework

Alternatively, you can build a Rust-based Lambda function declaratively using the [Serverless framework Rust plugin](https://github.com/softprops/serverless-rust).

A number of getting started Serverless application templates exist to get you up and running quickly

- a minimal [echo function](https://github.com/softprops/serverless-aws-rust) to demonstrate what the smallest Rust function setup looks like
- a minimal [http function](https://github.com/softprops/serverless-aws-rust-http) to demonstrate how to interface with API Gateway using Rust's native [http](https://crates.io/crates/http) crate (note this will be a git dependency until 0.2 is published)
- a combination [multi function service](https://github.com/softprops/serverless-aws-rust-multi) to demonstrate how to set up a services with multiple independent functions

Assuming your host machine has a relatively recent version of node, you [won't need to install any host-wide serverless dependencies](https://blog.npmjs.org/post/162869356040/introducing-npx-an-npm-package-runner). To get started, run the following commands to create a new lambda Rust application
and install project level dependencies.

```bash
$ npx serverless install \
  --url https://github.com/softprops/serverless-aws-rust \
  --name my-new-app \
  && cd my-new-app \
  && npm install --silent
```

Deploy it using the standard serverless workflow

```bash
# build, package, and deploy service to aws lambda
$ npx serverless deploy
```

Invoke it using serverless framework or a configured AWS integrated trigger source:

```bash
$ npx serverless invoke -f hello -d '{"foo":"bar"}'
```

#### Docker

Alternatively, you can build a Rust-based Lambda function in a [docker mirror of the AWS Lambda provided runtime with the Rust toolchain preinstalled](https://github.com/softprops/lambda-rust).

Running the following command will start a ephemeral docker container which will build your Rust application and produce a zip file containing its binary auto-renamed to `bootstrap` to meet the AWS Lambda's expectations for binaries under `target/lambda/release/{your-binary-name}.zip`, typically this is just the name of your crate if you are using the cargo default binary (i.e. `main.rs`)

```bash
# build and package deploy-ready artifact
$ docker run --rm \
    -v ${PWD}:/code \
    -v ${HOME}/.cargo/registry:/root/.cargo/registry \
    -v ${HOME}/.cargo/git:/root/.cargo/git \
    softprops/lambda-rust
```

With your application build and packaged, it's ready to ship to production. You can also invoke it locally to verify is behavior using the [lambdaci :provided docker container](https://hub.docker.com/r/lambci/lambda/) which is also a mirror of the AWS Lambda provided runtime with build dependencies omitted.

```bash
# start a docker container replicating the "provided" lambda runtime
# awaiting an event to be provided via stdin
$ unzip -o \
    target/lambda/release/{your-binary-name}.zip \
    -d /tmp/lambda && \
  docker run \
    -i -e DOCKER_LAMBDA_USE_STDIN=1 \
    --rm \
    -v /tmp/lambda:/var/task \
    lambci/lambda:provided

# provide an event payload via stdin (typically a json blob)

# Ctrl-D to yield control back to your function
```

## `lambda`

`lambda` is a library for authoring reliable and performant Rust-based AWS Lambda functions. At a high level, it provides a few major components:

- `Handler`, a trait that defines interactions between customer-authored code and this library.
- `lambda::run`, function that runs an `Handler`.

The function `handler_fn` converts a rust function or closure to `Handler`, which can then be run by `lambda::run`.

## AWS event objects

This project does not currently include Lambda event struct definitions though we [intend to do so in the future](https://github.com/awslabs/aws-lambda-rust-runtime/issues/12). Instead, the community-maintained [`aws_lambda_events`](https://crates.io/crates/aws_lambda_events) crate can be leveraged to provide strongly-typed Lambda event structs. You can create your own custom event objects and their corresponding structs as well.

## Custom event objects

To serialize and deserialize events and responses, we suggest using the use the [`serde`](https://github.com/serde-rs/serde) library. To receive custom events, annotate your structure with Serde's macros:

```rust
use serde::{Serialize, Deserialize};
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
