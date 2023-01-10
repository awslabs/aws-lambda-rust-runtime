# AWS Lambda Function that receives events from SQS

This example shows how to process events from an SQS queue using the partial batch failure feature.

_Important note:_ your lambda sqs trigger *needs* to be configured with partial batch response support
(the ` ReportBatchItemFailures` flag set to true), otherwise failed message will be not be reprocessed.
For more details see:
https://docs.aws.amazon.com/lambda/latest/dg/with-sqs.html#services-sqs-batchfailurereporting

## Build & Deploy

1. Install [cargo-lambda](https://github.com/cargo-lambda/cargo-lambda#installation)
2. Build the function with `cargo lambda build --release`
3. Deploy the function to AWS Lambda with `cargo lambda deploy --iam-role YOUR_ROLE`

## Build for ARM 64

Build the function with `cargo lambda build --release --arm64`