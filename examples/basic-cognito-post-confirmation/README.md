# Cognito Post Confirmation Request example

This example shows how to write a Lambda function in Rust to process Cognito's Post Confirmation requests.

This is a translation of the example in the AWS Docs to Rust: https://docs.aws.amazon.com/cognito/latest/developerguide/user-pool-lambda-post-confirmation.html#aws-lambda-triggers-post-confirmation-example

## Build & Deploy

1. Install [cargo-lambda](https://github.com/cargo-lambda/cargo-lambda#installation)
2. Build the function with `cargo lambda build --release`
3. Deploy the function to AWS Lambda with `cargo lambda deploy`

## Build for ARM 64

Build the function with `cargo lambda build --release --arm64`
