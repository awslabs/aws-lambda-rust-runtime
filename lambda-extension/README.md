# Runtime Extensions for AWS Lambda in Rust

[![Docs](https://docs.rs/lambda_extension/badge.svg)](https://docs.rs/lambda_extension)

**`lambda-extension`** is a library that makes it easy to write [AWS Lambda Runtime Extensions](https://docs.aws.amazon.com/lambda/latest/dg/using-extensions.html) in Rust. It also helps with using [Lambda Logs API](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-logs-api.html).

## Example extensions

### Simple extension

The code below creates a simple extension that's registered to every `INVOKE` and `SHUTDOWN` events.

```rust,no_run
use lambda_extension::{service_fn, Error, LambdaEvent, NextEvent};

async fn my_extension(event: LambdaEvent) -> Result<(), Error> {
    match event.next {
        NextEvent::Shutdown(_e) => {
            // do something with the shutdown event
        }
        NextEvent::Invoke(_e) => {
            // do something with the invoke event
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let func = service_fn(my_extension);
    lambda_extension::run(func).await
}

```

### Log processor extension

```rust,no_run
use lambda_extension::{service_fn, Error, Extension, LambdaLog, LambdaLogRecord, SharedService};
use tracing::info;

async fn handler(logs: Vec<LambdaLog>) -> Result<(), Error> {
    for log in logs {
        match log.record {
            LambdaLogRecord::Function(_record) => {
                // do something with the function log record
            },
            LambdaLogRecord::Extension(_record) => {
                // do something with the extension log record
            },
            },
            _ => (),
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let logs_processor = SharedService::new(service_fn(handler));

    Extension::new().with_logs_processor(logs_processor).run().await?;

    Ok(())
}

```

### Telemetry processor extension

```rust,no_run
use lambda_extension::{service_fn, Error, Extension, LambdaTelemetry, LambdaTelemetryRecord, SharedService};
use tracing::info;

async fn handler(events: Vec<LambdaTelemetry>) -> Result<(), Error> {
    for event in events {
        match event.record {
            LambdaTelemetryRecord::Function(record) => {
                // do something with the function log record
            },
            LambdaTelemetryRecord::PlatformInitStart {
                initialization_type: _,
                phase: _,
                runtime_version: _,
                runtime_version_arn: _,
            } => {
                // do something with the PlatformInitStart event
            },
            // more types of telemetry events are available
            _ => (),
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let telemetry_processor = SharedService::new(service_fn(handler));

    Extension::new().with_telemetry_processor(telemetry_processor).run().await?;

    Ok(())
}

```

## Deployment

Lambda extensions can be added to your functions either using [Lambda layers](https://docs.aws.amazon.com/lambda/latest/dg/using-extensions.html#using-extensions-config), or adding them to [containers images](https://docs.aws.amazon.com/lambda/latest/dg/using-extensions.html#invocation-extensions-images).

Regardless of how you deploy them, the extensions MUST be compiled against the same architecture that your lambda functions runs on.

### Building extensions

- Install [cargo-lambda](https://github.com/cargo-lambda/cargo-lambda#installation)

- Build the extension with:

```
cargo lambda build --release --extension
```

If you want to run the extension in ARM processors, add the `--arm64` flag to the previous command:

```
cargo lambda build --release --extension --arm64
```

This previous command will generate a binary file in `target/lambda/extensions` called `basic`. When the extension is registered with the [Runtime Extensions API](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-extensions-api.html#runtimes-extensions-api-reg), that's the name that the extension will be registered with. If you want to register the extension with a different name, you only have to rename this binary file and deploy it with the new name.

### Deploying extensions

- Make sure you have the right credentials in your terminal by running the AWS CLI configure command:

```
aws configure
```

- Deploy the extension as a layer with:

```
cargo lambda deploy --extension
```

