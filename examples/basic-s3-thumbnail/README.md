# AWS Lambda Function that uses S3

This example processes S3 events. If the event is a CREATE event,
it downloads the created file, generates a thumbnail from it
(it assumes that the file is an image) and uploads it to S3 into a bucket named
[original-bucket-name]-thumbs.

## Set up
1. Create a lambda function and upload the bootloader.zip
2. Go to aws services S3
3. Create a bucket, let's say with name bucketx
4. Create another bucket bucketx-thumbs
5. Got to the bucketx properties tab, event notifications
6. Create lambda event notification for "all object create event" and select your lambda function
7. Go to the lambda function, configuration and open the role name
8. Add "AmazonS3FullAccess" permission

## Test

1. Go to S3 and upload a png picture into bucketx. Beware to not have spaces or any special characters in the file name
2. Go to S3 bucketx-thumbs and check if an image is created there.


## Build & Deploy

1. Install [cargo-lambda](https://github.com/cargo-lambda/cargo-lambda#installation)
2. Build the function with `cargo lambda build --release` 
3. Deploy the function to AWS Lambda with `cargo lambda deploy --iam-role YOUR_ROLE`

## Build for ARM 64

Build the function with `cargo lambda build --release --arm64` 