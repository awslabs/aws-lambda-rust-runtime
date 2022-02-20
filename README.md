# Rust Runtime for AWS Lambda

[![Build Status](https://github.com/awslabs/aws-lambda-rust-runtime/workflows/Rust/badge.svg)](https://github.com/awslabs/aws-lambda-rust-runtime/actions)

This package makes it easy to run AWS Lambda Functions written in Rust. This workspace includes multiple crates:

- [![Docs](https://docs.rs/lambda_runtime/badge.svg)](https://docs.rs/lambda_runtime) **`lambda-runtime`** is a library that provides a Lambda runtime for applications written in Rust.
- [![Docs](https://docs.rs/lambda_http/badge.svg)](https://docs.rs/lambda_http) **`lambda-http`** is a library that makes it easy to write API Gateway proxy event focused Lambda functions in Rust.
- [![Docs](https://docs.rs/lambda-extension/badge.svg)](https://docs.rs/lambda-extension) **`lambda-extension`** is a library that makes it easy to write Lambda Runtime Extensions in Rust.
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

## Building and deploying your Lambda functions

There are currently multiple ways of building this package: manually with the AWS CLI, with [AWS SAM](https://github.com/aws/aws-sam-cli), and with the [Serverless framework](https://serverless.com/framework/).

### 1. Cross-compiling your Lambda functions

At the time of writing, the ability to cross compile to a different architecture is limited. For example, you might experience issues if you are compiling from a MacOS machine, or from a Linux machine with a different architecture (e.g. compiling to Arm64 from x86_64). The most robust way we've found is using [`cargo-zigbuild`](https://github.com/messense/cargo-zigbuild) to compile for the target architecture.

#### 1.1. Setup the cross-compilation environment

_You can skip this step if you are compiling for the same target as your host architecture (e.g. x86_64 Linux to x86_64 Linux), unless you're building for an Amazon Linux 1 runtime._

Run this script once to add your desired target:

```bash
# For Arm64 Lambda functions
rustup target add aarch64-unknown-linux-gnu
# For x86_64 Lambda functions
rustup target add x86_64-unknown-linux-gnu
```

Once this is done, install [Zig](https://ziglang.org/) using the instructions in their [installation guide](https://ziglang.org/learn/getting-started/#installing-zig), and install `cargo-zigbuild`:

```bash
cargo install cargo-zigbuild
```

#### 1.2. Build your Lambda functions

__Amazon Linux 2__

We recommend you to use Amazon Linux 2 runtimes (such as `provided.al2`) as much as possible for building Lambda functions in Rust. To build your Lambda functions for Amazon Linux 2 runtimes, run:

```bash
# Note: replace "aarch64" with "x86_64" if you are building for x86_64
cargo zigbuild --release --target aarch64-unknown-linux-gnu
```

__Amazon Linux 1__

Amazon Linux 1 uses glibc version 2.17, while Rust binaries need glibc version 2.18 or later by default. However, with `cargo-zigbuild`, you can specify a different version of glibc.

If you are building for Amazon Linux 1, or you want to support both Amazon Linux 2 and 1, run:

```
# Note: replace "aarch64" with "x86_64" if you are building for x86_64
cargo zigbuild --release --target aarch64-unknown-linux-gnu.2.17
```

### 2. Deploying the binary to AWS Lambda

For [a custom runtime](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-custom.html), AWS Lambda looks for an executable called `bootstrap` in the deployment package zip. Rename the generated executable to `bootstrap` and add it to a zip archive.

__Note__: Depending on the target you used above, you'll find the provided basic executable under the corresponding directory. For example, if you are building the aarch64-unknown-linux-gnu as your target, it will be under `./target/aarch64-unknown-linux-gnu/release/`.

#### 2.1. Deploying with the AWS CLI

First, you will need to create a ZIP archive of your Lambda function. For example, if you are using the `basic` example and aarch64-unknown-linux-gnu as your target, you can run:

```bash
cp ./target/aarch64-unknown-linux-gnu/release/examples/basic ./bootstrap && zip lambda.zip bootstrap && rm bootstrap
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

#### 2.2. AWS Serverless Application Model (SAM)

You can use Lambda functions built in Rust with the [AWS Serverless Application Model (SAM)](https://aws.amazon.com/serverless/sam/). To do so, you will need to install the [AWS SAM CLI](https://github.com/aws/aws-sam-cli), which will help you package and deploy your Lambda functions in your AWS account.

You will need to create a `template.yaml` file containing your desired infrastructure in YAML. Here is an example with a single Lambda function:

```yaml
AWSTemplateFormatVersion: '2010-09-09'
Transform: AWS::Serverless-2016-10-31

Resources:
  HelloWorldFunction:
    Type: AWS::Serverless::Function
    Properties:
      MemorySize: 128
      Architectures: ["aarch64"]
      Handler: bootstrap
      Runtime: provided.al2
      Timeout: 5
      CodeUri: build/

Outputs:
  FunctionName:
    Value: !Ref HelloWorldFunction
    Description: Name of the Lambda function
```

After building your function, you will also need to store the binary as `bootstrap` in a dedicated directory for that function (e.g. `build/bootstrap`):

```bash
mkdir build
cp ./target/aarch64-unknown-linux-gnu/release/examples/basic ./build/bootstrap
```

You can then deploy your Lambda function using the AWS SAM CLI:

```bash
sam deploy --guided
```

At the end, `sam` will output the actual Lambda function name. You can use this name to invoke your function:

```bash
$ aws lambda invoke
  --cli-binary-format raw-in-base64-out \
  --function-name HelloWorldFunction-XXXXXXXX \ # Replace with the actual function name
  --payload '{"command": "Say Hi!"}' \
  output.json
$ cat output.json  # Prints: {"msg": "Command Say Hi! executed."}
```

#### 2.3. Serverless Framework

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

### Docker

Alternatively, you can build a Rust-based Lambda function in a [docker mirror of the AWS Lambda provided runtime with the Rust toolchain preinstalled](https://github.com/rustserverless/lambda-rust).

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

## `lambda_runtime`

`lambda_runtime` is a library for authoring reliable and performant Rust-based AWS Lambda functions. At a high level, it provides `lambda_runtime::run`, a function that runs a `tower::Service<LambdaEvent>`.

To write a function that will handle request, you need to pass it through `service_fn`, which will convert your function into a `tower::Service<LambdaEvent>`, which can then be run by `lambda_runtime::run`.

### AWS event objects

This project does not currently include Lambda event struct definitions though we [intend to do so in the future](https://github.com/awslabs/aws-lambda-rust-runtime/issues/12). Instead, the community-maintained [`aws_lambda_events`](https://crates.io/crates/aws_lambda_events) crate can be leveraged to provide strongly-typed Lambda event structs. You can create your own custom event objects and their corresponding structs as well.

### Custom event objects

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
