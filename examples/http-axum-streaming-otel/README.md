# AWS Lambda Function example

This example shows how to build a **streaming HTTP response** with `Axum` and
run it on AWS Lambda using a custom runtime with OpenTelemetry (OTel) support.

Tracing data is exported as console log entries visible in CloudWatch. Note that
CloudWatch assigns a `Timestamp` to each log entry based on when it receives the
data (batch exported). To see when work actually occurred, look at the span's
event attributes, which include the precise local timestamps of those events.

## Build & Deploy

1. Install
   [cargo-lambda](https://github.com/cargo-lambda/cargo-lambda#installation)
2. Build the function with `cargo lambda build --release`
3. Deploy the function to AWS Lambda with:
   - `cargo lambda deploy --enable-function-url --iam-role YOUR_ROLE --env-var
USE_NUMBERS=0` to stream words
   - `cargo lambda deploy --enable-function-url --iam-role YOUR_ROLE --env-var
USE_NUMBERS=1` to stream numbers.
4. Enable Lambda streaming response on Lambda console: change the function url's
   invoke mode to `RESPONSE_STREAM`
5. Verify the function works: `curl -N <function-url>`. The results should be
   streamed back with 0.5 second pause between each word.

## Build for ARM 64

Build the function with `cargo lambda build --release --arm64`
