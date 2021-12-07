# Runtime Extensions for AWS Lambda in Rust

[![Docs](https://docs.rs/lambda_extension/badge.svg)](https://docs.rs/lambda_extension)

**`lambda-extension`** is a library that makes it easy to write AWS Lambda Runtime Extensions in Rust.

## Example extension

The code below creates a simple extension that's registered to every `INVOKE` and `SHUTDOWN` events, and logs them in CloudWatch.

```rust,no_run
use lambda_extension::{extension_fn, Error, NextEvent};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use tracing::info;

async fn log_extension(event: NextEvent) -> Result<(), Error> {
    match event {
        NextEvent::Shutdown(event) => {
            info!("{}", event);
        }
        NextEvent::Invoke(event) => {
            info!("{}", event);
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();

    let func = extension_fn(log_extension);
    lambda_extension::run(func).await
}
```

## Deployment

Lambda extensions can be added to your functions either using [Lambda layers](https://docs.aws.amazon.com/lambda/latest/dg/using-extensions.html#using-extensions-config), or adding them to [containers images](https://docs.aws.amazon.com/lambda/latest/dg/using-extensions.html#invocation-extensions-images). Regardless of how you deploy them, the extensions MUST be compiled against the same architecture that your lambda functions runs on. The only two architectures that AWS Lambda supports are `aarch64-unknown-linux-gnu` for ARM functions, and `x86_64-unknown-linux-gnu` for x86 functions.

### Building extensions

Once you've decided which target you'll use, you can install it by running the next `rustup` command:

```bash
$ rustup target add x86_64-unknown-linux-gnu
```

Then, you can compile the extension against that target:

```bash
$ cargo build -p lambda_extension --example basic --release --target x86_64-unknown-linux-gnu
```

This previous command will generate a binary file in `target/x86_64-unknown-linux-gnu/release/examples` called `basic`. When the extension is registered with the [Runtime Extensions API](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-extensions-api.html#runtimes-extensions-api-reg), that's the name that the extension will be registered with. If you want to register the extension with a different name, you only have to rename this binary file and deploy it with the new name.