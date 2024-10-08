# AWS Lambda Function Error Handling with several popular error crates.

This example shows how to use external error types like `anyhow::Error`, `eyre::Report`, and `miette::Report`.

To use the integrations with these crates, you need to enable to respective feature flag in the runtime which provides the implemetation of `into_diagnostic` for specific error types provided by these crates.

## Build & Deploy

1. Install [cargo-lambda](https://github.com/cargo-lambda/cargo-lambda#installation)
2. Build the function with `cargo lambda build --release`
3. Deploy the function to AWS Lambda with `cargo lambda deploy --iam-role YOUR_ROLE`

## Build for ARM 64

Build the function with `cargo lambda build --release --arm64`
