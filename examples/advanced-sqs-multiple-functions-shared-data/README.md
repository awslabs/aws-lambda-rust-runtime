# AWS Lambda Function example

## Build & Deploy

1. Install [cargo-lambda](https://github.com/cargo-lambda/cargo-lambda#installation)
2. Build the function with `cargo lambda build --release`
3. Deploy the function to AWS Lambda with `cargo lambda deploy --iam-role YOUR_ROLE`

## Build for ARM 64

Build the function with `cargo lambda build --release --arm64`

## Add the SQS trigger to the consumer function

You can use aws-cli to create an event source mapping:

`aws lambda create-event-source-mapping \
--function-name consumer \
--region <region> \
--event-source-arn <your-SQS-queue-ARN> \
--batch-size 1`

