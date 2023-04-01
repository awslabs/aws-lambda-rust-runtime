# AWS Lambda Function that uses S3

This example processes S3 events. If the event is a CREATE event,
it downloads the created file, generates a thumbnail from it
(it assumes that the file is an image) and uploads it to S3 into a bucket named
[original-bucket-name]-thumbs.

## Build & Deploy

1. Install [cargo-lambda](https://github.com/cargo-lambda/cargo-lambda#installation)
2. Build the function with `cargo lambda build --release` 
3. Deploy the function to AWS Lambda with `cargo lambda deploy --iam-role YOUR_ROLE`

## Build for ARM 64

Build the function with `cargo lambda build --release --arm64` 