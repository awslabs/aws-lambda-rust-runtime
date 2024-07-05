# AWS Lambda Function Error handling example

This example shows how to implement the `Diagnostic` trait to return a specific `error_type` in the Lambda error response. If you don't use the `error_type` field, you don't need to implement `Diagnostic`, the type will be generated based on the error type name.

## Build & Deploy

1. Install [cargo-lambda](https://github.com/cargo-lambda/cargo-lambda#installation)
2. Build the function with `cargo lambda build --release`
3. Deploy the function to AWS Lambda with `cargo lambda deploy --iam-role YOUR_ROLE`

## Build for ARM 64

Build the function with `cargo lambda build --release --arm64`
