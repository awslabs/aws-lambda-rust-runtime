# Rust Runtime for AWS Lambda

[![Build Status](https://github.com/awslabs/aws-lambda-rust-runtime/workflows/Rust/badge.svg)](https://github.com/awslabs/aws-lambda-rust-runtime/actions)

This package makes it easy to run AWS Lambda Functions written in Rust. This workspace includes multiple crates:

- [![Docs](https://docs.rs/lambda_runtime/badge.svg)](https://docs.rs/lambda_runtime) **`lambda-runtime`** is a library that provides a Lambda runtime for applications written in Rust.
- [![Docs](https://docs.rs/lambda_http/badge.svg)](https://docs.rs/lambda_http) **`lambda-http`** is a library that makes it easy to write API Gateway proxy event focused Lambda functions in Rust.
- [![Docs](https://docs.rs/lambda-extension/badge.svg)](https://docs.rs/lambda-extension) **`lambda-extension`** is a library that makes it easy to write Lambda Runtime Extensions in Rust.
- [![Docs](https://docs.rs/aws_lambda_events/badge.svg)](https://docs.rs/aws_lambda_events) **`lambda-events`** is a library with strongly-typed Lambda event structs in Rust.
- [![Docs](https://docs.rs/lambda_runtime_api_client/badge.svg)](https://docs.rs/lambda_runtime_api_client) **`lambda-runtime-api-client`** is a shared library between the lambda runtime and lambda extension libraries that includes a common API client to talk with the AWS Lambda Runtime API.

The Rust runtime client is an experimental package. It is subject to change and intended only for evaluation purposes.

## Getting started

The easiest way to start writing Lambda functions with Rust is by using [Cargo Lambda](https://www.cargo-lambda.info/), a related project. Cargo Lambda is a Cargo plugin, or subcommand, that provides several commands to help you in your journey with Rust on AWS Lambda.

The preferred way to install Cargo Lambda is by using a package manager.

1- Use Homebrew on [MacOS](https://brew.sh/):

```bash
brew tap cargo-lambda/cargo-lambda
brew install cargo-lambda
```

2- Use [Scoop](https://scoop.sh/) on Windows:

```bash
scoop bucket add cargo-lambda https://github.com/cargo-lambda/scoop-cargo-lambda
scoop install cargo-lambda/cargo-lambda
```

Or PiP on any system with Python 3 installed:

```bash
pip3 install cargo-lambda
```

See other installation options in [the Cargo Lambda documentation](https://www.cargo-lambda.info/guide/installation.html).

### Your first function

To create your first function, run Cargo Lambda with the [subcommand `new`](https://www.cargo-lambda.info/commands/new.html). This command will generate a Rust package with the initial source code for your function:

```bash
cargo lambda new YOUR_FUNCTION_NAME
```

### Example function

If you'd like to manually create your first function, the code below shows you a simple function that receives an event with a `firstName` field and returns a message to the caller.

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

If you already have Cargo Lambda installed in your machine, run the next command to build your function:

```bash
cargo lambda build --release
```

There are other ways of building your function: manually with the AWS CLI, with [AWS SAM](https://github.com/aws/aws-sam-cli), and with the [Serverless framework](https://serverless.com/framework/).

### 1. Cross-compiling your Lambda functions

By default, Cargo Lambda builds your functions to run on x86_64 architectures. If you'd like to use a different architecture, use the options described below.

#### 1.2. Build your Lambda functions

__Amazon Linux 2__

We recommend you to use Amazon Linux 2 runtimes (such as `provided.al2`) as much as possible for building Lambda functions in Rust. To build your Lambda functions for Amazon Linux 2 runtimes, run:

```bash
cargo lambda build --release --arm64
```

__Amazon Linux 1__

Amazon Linux 1 uses glibc version 2.17, while Rust binaries need glibc version 2.18 or later by default. However, with Cargo Lambda, you can specify a different version of glibc.

If you are building for Amazon Linux 1, or you want to support both Amazon Linux 2 and 1, run:

```bash
cargo lambda build --release --target aarch64-unknown-linux-gnu.2.17
```
> **Note**
> Replace "aarch64" with "x86_64" if you are building for x86_64
### 2. Deploying the binary to AWS Lambda

For [a custom runtime](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-custom.html), AWS Lambda looks for an executable called `bootstrap` in the deployment package zip. Rename the generated executable to `bootstrap` and add it to a zip archive.

You can find the `bootstrap` binary for your function under the `target/lambda` directory.

#### 2.2. Deploying with Cargo Lambda

Once you've built your code with one of the options described earlier, use the `deploy` subcommand to upload your function to AWS:

```bash
cargo lambda deploy \
  --iam-role arn:aws:iam::XXXXXXXXXXXXX:role/your_lambda_execution_role
```

> **Warning**
> Make sure to replace the execution role with an existing role in your account!

This command will create a Lambda function with the same name of your rust package. You can change the name
of the function by adding the argument at the end of the command:

```bash
cargo lambda deploy \
  --iam-role arn:aws:iam::XXXXXXXXXXXXX:role/your_lambda_execution_role \
  my-first-lambda-function
```

> **Note**
> See other deployment options in [the Cargo Lambda documentation](https://www.cargo-lambda.info/commands/deploy.html).

You can test the function with the [invoke subcommand](https://www.cargo-lambda.info/commands/invoke.html):

```bash
cargo lambda invoke --remote \
  --data-ascii '{"command": "hi"}' \
  --output-format json \
  my-first-lambda-function
```

> **Note**
> CLI commands in the examples use Linux/MacOS syntax. For different shells like Windows CMD or PowerShell, modify syntax when using nested quotation marks like `'{"command": "hi"}'`. Escaping with a backslash may be necessary. See [AWS CLI Reference](https://docs.amazonaws.cn/en_us/cli/latest/userguide/cli-usage-parameters-quoting-strings.html#cli-usage-parameters-quoting-strings-containing) for more information.

#### 2.2. Deploying with the AWS CLI

You can also use the AWS CLI to deploy your Rust functions. First, you will need to create a ZIP archive of your  function. Cargo Lambda can do that for you automatically when it builds your binary if you add the `output-format` flag:

```bash
cargo lambda build --release --arm64 --output-format zip
```

You can find the resulting zip file in `target/lambda/YOUR_PACKAGE/bootstrap.zip`. Use that file path to deploy your function with the [AWS CLI](https://aws.amazon.com/cli/):

```bash
$ aws lambda create-function --function-name rustTest \
  --handler bootstrap \
  --zip-file fileb://./target/lambda/basic/bootstrap.zip \
  --runtime provided.al2 \ # Change this to provided.al if you would like to use Amazon Linux 1.
  --role arn:aws:iam::XXXXXXXXXXXXX:role/your_lambda_execution_role \
  --environment Variables={RUST_BACKTRACE=1} \
  --tracing-config Mode=Active
```

> **Warning**
> Make sure to replace the execution role with an existing role in your account!

You can now test the function using the AWS CLI or the AWS Lambda console

```bash
$ aws lambda invoke
  --cli-binary-format raw-in-base64-out \
  --function-name rustTest \
  --payload '{"command": "Say Hi!"}' \
  output.json
$ cat output.json  # Prints: {"msg": "Command Say Hi! executed."}
```

> **Note** 
> `--cli-binary-format raw-in-base64-out` is a required argument when using the AWS CLI version 2. [More Information](https://docs.aws.amazon.com/cli/latest/userguide/cliv2-migration.html#cliv2-migration-binaryparam)

#### 2.3. AWS Serverless Application Model (SAM)

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
      Architectures: ["arm64"]
      Handler: bootstrap
      Runtime: provided.al2
      Timeout: 5
      CodeUri: target/lambda/basic/

Outputs:
  FunctionName:
    Value: !Ref HelloWorldFunction
    Description: Name of the Lambda function
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

#### 2.4. Serverless Framework

Alternatively, you can build a Rust-based Lambda function declaratively using the [Serverless framework Rust plugin](https://github.com/softprops/serverless-rust).

A number of getting started Serverless application templates exist to get you up and running quickly:

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

Deploy it using the standard serverless workflow:

```bash
# build, package, and deploy service to aws lambda
$ npx serverless deploy
```

Invoke it using serverless framework or a configured AWS integrated trigger source:

```bash
npx serverless invoke -f hello -d '{"foo":"bar"}'
```

#### 2.5. Docker

Alternatively, you can build a Rust-based Lambda function in a [docker mirror of the AWS Lambda provided runtime with the Rust toolchain preinstalled](https://github.com/rust-serverless/lambda-rust).

Running the following command will start an ephemeral docker container, which will build your Rust application and produce a zip file containing its binary auto-renamed to `bootstrap` to meet the AWS Lambda's expectations for binaries under `target/lambda_runtime/release/{your-binary-name}.zip`. Typically, this is just the name of your crate if you are using the cargo default binary (i.e. `main.rs`):

```bash
# build and package deploy-ready artifact
$ docker run --rm \
    -v ${PWD}:/code \
    -v ${HOME}/.cargo/registry:/root/.cargo/registry \
    -v ${HOME}/.cargo/git:/root/.cargo/git \
    rustserverless/lambda-rust
```

With your application build and packaged, it's ready to ship to production. You can also invoke it locally to verify is behavior using the [lambci :provided docker container](https://hub.docker.com/r/lambci/lambda/), which is also a mirror of the AWS Lambda provided runtime with build dependencies omitted:

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

## Local development and testing

### Testing your code with unit and integration tests

AWS Lambda events are plain structures deserialized from JSON objects.
If your function handler uses the standard runtime, you can use `serde` to deserialize
your text fixtures into the structures, and call your handler directly:

```rust,no_run
#[test]
fn test_my_lambda_handler() {
  let input = serde_json::from_str("{\"command\": \"Say Hi!\"}").expect("failed to parse event");
  let context = lambda_runtime::Context::default();

  let event = lambda_runtime::LambdaEvent::new(input, context);

  my_lambda_handler(event).await.expect("failed to handle event");
}
```

If you're using `lambda_http` to receive HTTP events, you can also create `http_lambda::Request`
structures from plain text fixtures:

```rust,no_run
#[test]
fn test_my_lambda_handler() {
  let input = include_str!("apigw_proxy_request.json");

  let request = lambda_http::request::from_str(input)
    .expect("failed to create request");

  let response = my_lambda_handler(request).await.expect("failed to handle request");
}
```

### Cargo Lambda

[Cargo Lambda](https://www.cargo-lambda.info) provides a local server that emulates the AWS Lambda control plane. This server works on Windows, Linux, and MacOS. In the root of your Lambda project. You can run the following subcommand to compile your function(s) and start the server.

```bash
cargo lambda watch -a 127.0.0.1 -p 9001
```

Now you can use the `cargo lambda invoke` to send requests to your function. For example:

```bash
cargo lambda invoke <lambda-function-name> --data-ascii '{ "command": "hi" }'
```

Running the command on a HTTP function (Function URL, API Gateway, etc) will require you to use the appropriate scheme. You can find examples of these schemes [here](https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/lambda-http/tests/data). Otherwise, you will be presented with the following error.

```rust,no_run
Error: serde_json::error::Error

  × data did not match any variant of untagged enum LambdaRequest
```

An simpler alternative is to cURL the following endpoint based on the address and port you defined. For example:

```bash
curl -v -X POST \
  'http://127.0.0.1:9001/lambda-url/<lambda-function-name>/' \
  -H 'content-type: application/json' \
  -d '{ "command": "hi" }'
```

> **Warning** 
> Do not remove the `content-type` header. It is necessary to instruct the function how to deserialize the request body.

You can read more about how [cargo lambda watch](https://www.cargo-lambda.info/commands/watch.html) and [cargo lambda invoke](https://www.cargo-lambda.info/commands/invoke.html) work on the project's [documentation page](https://www.cargo-lambda.info).

### Lambda Debug Proxy

Lambdas can be run and debugged locally using a special [Lambda debug proxy](https://github.com/rimutaka/lambda-debug-proxy) (a non-AWS repo maintained by @rimutaka), which is a Lambda function that forwards incoming requests to one AWS SQS queue and reads responses from another queue. A local proxy running on your development computer reads the queue, calls your Lambda locally and sends back the response. This approach allows debugging of Lambda functions locally while being part of your AWS workflow. The Lambda handler code does not need to be modified between the local and AWS versions.

## AWS event objects

This project does not currently include Lambda event struct definitions. Instead, the community-maintained [`aws_lambda_events`](https://crates.io/crates/aws_lambda_events) crate can be leveraged to provide strongly-typed Lambda event structs. You can create your own custom event objects and their corresponding structs as well.

### Custom event objects

To serialize and deserialize events and responses, we suggest using the [`serde`](https://github.com/serde-rs/serde) library. To receive custom events, annotate your structure with Serde's macros:

```rust,no_run
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

## Feature flags in lambda_http

`lambda_http` is a wrapper for HTTP events coming from three different services, Amazon Load Balancer (ALB), Amazon Api Gateway (APIGW), and AWS Lambda Function URLs. Amazon Api Gateway can also send events from three different endpoints, REST APIs, HTTP APIs, and WebSockets. `lambda_http` transforms events from all these sources into native `http::Request` objects, so you can incorporate Rust HTTP semantics into your Lambda functions.

By default, `lambda_http` compiles your function to support any of those services. This increases the compile time of your function because we have to generate code for all the sources. In reality, you'll usually put a Lambda function only behind one of those sources. You can choose which source to generate code for with feature flags.

The available features flags for `lambda_http` are the following:

- `alb`: for events coming from [Amazon Elastic Load Balancer](https://aws.amazon.com/elasticloadbalancing/).
- `apigw_rest`: for events coming from [Amazon API Gateway Rest APIs](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-rest-api.html).
- `apigw_http`: for events coming from [Amazon API Gateway HTTP APIs](https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api.html) and [AWS Lambda Function URLs](https://docs.aws.amazon.com/lambda/latest/dg/lambda-urls.html).
- `apigw_websockets`: for events coming from [Amazon API Gateway WebSockets](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-websocket-api.html).

If you only want to support one of these sources, you can disable the default features, and enable only the source that you care about in your package's `Cargo.toml` file. Substitute the dependency line for `lambda_http` for the snippet below, changing the feature that you want to enable:

```toml
[dependencies.lambda_http]
version = "0.5.3"
default-features = false
features = ["apigw_rest"]
```

This will make your function compile much faster.

## Supported Rust Versions (MSRV)

The AWS Lambda Rust Runtime requires a minimum of Rust 1.64, and is not guaranteed to build on compiler versions earlier than that.

## Security

See [CONTRIBUTING](CONTRIBUTING.md#security-issue-notifications) for more information.

## License

This project is licensed under the Apache-2.0 License.
