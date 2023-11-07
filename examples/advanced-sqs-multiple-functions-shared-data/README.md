# AWS Lambda Function example

## Build & Deploy

1. Install [cargo-lambda](https://github.com/cargo-lambda/cargo-lambda#installation)
2. Build the function with `cargo lambda build --release`
4. Make sure to edit the QUEUE_URL env variable in producer/Cargo.toml
3. Deploy boths functions to AWS Lambda with

`cargo lambda deploy consumer --iam-role YOUR_ROLE`

`cargo lambda deploy producer --iam-role YOUR_ROLE`

## Build for ARM 64

Build the function with `cargo lambda build --release --arm64`

## Add the SQS trigger to the consumer function

You can use aws-cli to create an event source mapping:

```bash
aws lambda create-event-source-mapping \
--function-name consumer \
--region <region> \
--event-source-arn <your-SQS-queue-ARN> \
--batch-size 1
```