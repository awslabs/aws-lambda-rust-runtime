# AWS Lambda Function Error Handling With `anyhow` Crate Example

This example shows how to use external error types like `anyhow::Error`.

## Build & Deploy

1. Install [cargo-lambda](https://github.com/cargo-lambda/cargo-lambda#installation)
2. Build the function with `cargo lambda build --release`
3. Deploy the function to AWS Lambda with `cargo lambda deploy --iam-role YOUR_ROLE`

## Build for ARM 64

Build the function with `cargo lambda build --release --arm64`
