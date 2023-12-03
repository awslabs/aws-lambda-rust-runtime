# AWS Lambda runtime + internal extension example

This example demonstrates how to build an AWS Lambda function that includes a
[Lambda internal extension](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-extensions-api.html).
Unlike external extensions that run as separate processes, an internal extension runs within the
main runtime process.

One use case for internal extensions is to flush logs or telemetry data after the Lambda runtime
handler has finished processing an event but before the execution environment is frozen awaiting the
arrival of the next event. Without an explicit flush, telemetry data may never be sent since the
execution environment will remain frozen and eventually be terminated if no additional events arrive.

Note that for
[synchronous](https://docs.aws.amazon.com/lambda/latest/dg/invocation-sync.html) Lambda invocations
(e.g., via
[Amazon API Gateway](https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-lambda-integrations.html)),
the Lambda service returns the response to the caller immediately. Extensions may continue to run
without introducing an observable delay.

## Build & Deploy

1. Install [cargo-lambda](https://github.com/cargo-lambda/cargo-lambda#installation)
2. Build the extension with `cargo lambda build --release`
3. Deploy the function to AWS Lambda with `cargo lambda deploy --iam-role YOUR_ROLE`

The last command will give you an ARN for the extension layer that you can use in your functions.

## Build for ARM 64

Build the extension with `cargo lambda build --release --arm64`
