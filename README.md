# Rust Runtime for AWS Lambda

[![Build Status](https://github.com/awslabs/aws-lambda-rust-runtime/workflows/Rust/badge.svg)](https://github.com/awslabs/aws-lambda-rust-runtime/actions)

This package makes it easy to run AWS Lambda Functions written in Rust. This workspace includes multiple crates:

- [![Docs](https://docs.rs/lambda_runtime/badge.svg)](https://docs.rs/lambda_runtime) **`lambda-runtime`** is a library that provides a Lambda runtime for applications written in Rust.
- [![Docs](https://docs.rs/lambda_http/badge.svg)](https://docs.rs/lambda_http) **`lambda-http`** is a library that makes it easy to write API Gateway proxy event focused Lambda functions in Rust.
- [![Docs](https://docs.rs/lambda_extension/badge.svg)](https://docs.rs/lambda_extension) **`lambda-extension`** is a library that makes it easy to write Lambda Runtime Extensions in Rust.
- [![Docs](https://docs.rs/lambda_runtime_api_client/badge.svg)](https://docs.rs/lambda_runtime_api_client) **`lambda-runtime-api-client`** is a shared library between the lambda runtime and lambda extension libraries that includes a common API client to talk with the AWS Lambda Runtime API.


## Example function

The code below creates a simple function that receives an event with a `firstName` field and returns a message to the caller. Notice: this crate is tested against latest stable Rust.

```rust,no_run
use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let (event, _context) = event.into_parts();
    let first_name = event["firstName"].as_str().unwrap_or("world");

    Ok(json!({ "message": format!("Hello, {}!", first_name) }))
}
```

### Deployment

There are currently multiple ways of building this package: manually with the AWS CLI, and with the [Serverless framework](https://serverless.com/framework/).

#### AWS CLI

To deploy the basic sample as a Lambda function using the [AWS CLI](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-welcome.html), we first need to manually build it with [`cargo`](https://doc.rust-lang.org/cargo/). Due to a few differences in dependencies, the process for building for Amazon Linux 2 is slightly different than building for Amazon Linux.

**Building for Amazon Linux 2**

Decide which target you'd like to use. For ARM Lambda Functions you'll want to use `aarch64-unknown-linux-gnu` and for x86 Lambda Functions you'll want to use `x86_64-unknown-linux-gnu`.

Run this script once to add your desired target, in this example we'll use x86:

```bash
$ rustup target add x86_64-unknown-linux-gnu
```

Compile one of the examples as a _release_ with a specific _target_ for deployment to AWS:

```bash
$ cargo build -p lambda_runtime --example basic --release --target x86_64-unknown-linux-gnu
```

_Building on MacOS Using Docker_

At the time of writing, the ability to cross compile to x86 or aarch64 AL2 on MacOS is limited. The most robust way we've found is using Docker to produce the artifacts for you. This guide will work for both Intel and Apple Silicon MacOS and requires that you have set up Docker correctly for either architecture. [See here for a guide on how to do this.](https://docs.docker.com/desktop/mac/install/)

The following command will pull the [official Rust Docker Image](https://hub.docker.com/_/rust) for a given architecture you plan to use in Lambda and use it to run any cargo commands you need, such as build.

```bash
$ LAMBDA_ARCH="linux/arm64" # set this to either linux/arm64 for ARM functions, or linux/amd64 for x86 functions.
$ RUST_TARGET="aarch64-unknown-linux-gnu" # corresponding with the above, set this to aarch64 or x86_64 -unknown-linux-gnu for ARM or x86 functions.
$ RUST_VERSION="latest" # Set this to a specific version of rust you want to compile for, or to latest if you want the latest stable version.
$ docker run \
  --platform ${LAMBDA_ARCH} \
  --rm --user "$(id -u)":"$(id -g)" \
  -v "${PWD}":/usr/src/myapp -w /usr/src/myapp rust:${RUST_VERSION} \
  cargo build -p lambda_runtime --example basic --release --target ${RUST_TARGET} # This line can be any cargo command
```

In short, the above command does the following:

1. Gives the current user ownership of the artifacts produced by the cargo run.
2. Mounts the current working directory as a volume within the pulled Docker image.
3. Pulls a given Rust Docker image for a given platform.
4. Executes the command on the line beginning with `cargo` within the project directory within the image.

It is important to note that build artifacts produced from the above command can be found under the expected `target/` directory in the project after build.

**Building for Amazon Linux 1**

Run this script once to add the new target:
```bash
$ rustup target add x86_64-unknown-linux-musl
```

* **Note:** If you are running on Mac OS you'll need to install the linker for the target platform. You do this using the `musl-cross` tap from [Homebrew](https://brew.sh/) which provides a complete cross-compilation toolchain for Mac OS. Once `musl-cross` is installed we will also need to inform cargo of the newly installed linker when building for the `x86_64-unknown-linux-musl` platform.


```bash
$ brew install filosottile/musl-cross/musl-cross
$ mkdir .cargo
$ echo $'[target.x86_64-unknown-linux-musl]\nlinker = "x86_64-linux-musl-gcc"' > .cargo/config
```

Compile one of the examples as a _release_ with a specific _target_ for deployment to AWS:
```bash
$ cargo build -p lambda_runtime --example basic --release --target x86_64-unknown-linux-musl
```

**Uploading the Resulting Binary to Lambda**

For [a custom runtime](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-custom.html), AWS Lambda looks for an executable called `bootstrap` in the deployment package zip. Rename the generated `basic` executable to `bootstrap` and add it to a zip archive.

NOTE: Depending on the target you used above, you'll find the provided basic executable under the corresponding directory. In the following example, we've compiled for x86_64-unknown-linux-gnu.

```bash
$ cp ./target/x86_64-unknown-linux-gnu/release/examples/basic ./bootstrap && zip lambda.zip bootstrap && rm bootstrap
```

Now that we have a deployment package (`lambda.zip`), we can use the [AWS CLI](https://aws.amazon.com/cli/) to create a new Lambda function. Make sure to replace the execution role with an existing role in your account!

```bash
$ aws lambda create-function --function-name rustTest \
  --handler doesnt.matter \
  --zip-file fileb://./lambda.zip \
  --runtime provided.al2 \ # Change this to provided.al if you would like to use Amazon Linux 1.
  --role arn:aws:iam::XXXXXXXXXXXXX:role/your_lambda_execution_role \
  --environment Variables={RUST_BACKTRACE=1} \
  --tracing-config Mode=Active
```

You can now test the function using the AWS CLI or the AWS Lambda console

```bash
$ aws lambda invoke
  --cli-binary-format raw-in-base64-out \
  --function-name rustTest \
  --payload '{"command": "Say Hi!"}' \
  output.json
$ cat output.json  # Prints: {"msg": "Command Say Hi! executed."}
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

Alternatively, you can build a Rust-based Lambda function in a [docker mirror of the AWS Lambda provided runtime with the Rust toolchain preinstalled](https://github.com/    rustserverless/lambda-rust).

Running the following command will start a ephemeral docker container which will build your Rust application and produce a zip file containing its binary auto-renamed to `bootstrap` to meet the AWS Lambda's expectations for binaries under `target/lambda_runtime/release/{your-binary-name}.zip`, typically this is just the name of your crate if you are using the cargo default binary (i.e. `main.rs`)

```bash
# build and package deploy-ready artifact
$ docker run --rm \
    -v ${PWD}:/code \
    -v ${HOME}/.cargo/registry:/root/.cargo/registry \
    -v ${HOME}/.cargo/git:/root/.cargo/git \
    rustserverless/lambda-rust
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

### Debugging

Lambdas can be run and debugged locally using a special [Lambda debug proxy](https://github.com/rimutaka/lambda-debug-proxy) (a non-AWS repo maintained by @rimutaka), which is a Lambda function that forwards incoming requests to one AWS SQS queue and reads responses from another queue. A local proxy running on your development computer reads the queue, calls your lambda locally and sends back the response. This approach allows debugging of Lambda functions locally while being part of your AWS workflow. The lambda handler code does not need to be modified between the local and AWS versions.

## `lambda`

`lambda_runtime` is a library for authoring reliable and performant Rust-based AWS Lambda functions. At a high level, it provides `lambda_runtime::run`, a function that runs a `tower::Service<LambdaEvent>`.

To write a function that will handle request, you need to pass it through `service_fn`, which will convert your function into a `tower::Service<LambdaEvent>`, which can then be run by `lambda_runtime::run`.

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

## Supported Rust Versions (MSRV)

The AWS Lambda Rust Runtime requires a minimum of Rust 1.54, and is not guaranteed to build on compiler versions earlier than that.

## Security

See [CONTRIBUTING](CONTRIBUTING.md#security-issue-notifications) for more information.

## License

This project is licensed under the Apache-2.0 License.
