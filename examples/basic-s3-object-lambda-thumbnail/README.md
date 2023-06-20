# AWS S3 Object Lambda Function

It uses a GetObject event and it returns with a thumbnail instead of the real
object from the S3 bucket.
The thumbnail was tested only witn PNG files.

## Build & Deploy

1. Install [cargo-lambda](https://github.com/cargo-lambda/cargo-lambda#installation)
2. Build the function with `cargo lambda build --release --arm64 --output-format zip`
3. Upload the bootstrap.zip file from the directory:`target/lambda/basic-s3-object-lambda-thumbnail/`

## Setup on AWS S3

1. You need a bucket and upload a PNG file to that bucket
2. Set Access Point for that bucket
3. Set Object Lambda Access Point for the access point and use the uploaded lambda function as a transformer

## Set Up on AWS Lambda

0. Click on Code tab
1. Runtime settings - runtime: Custom runtime on Amazon Linux 2
2. Runtime settings - Architecture: arm64

## Set Up on AWS IAM

1. Click on Roles
2. Search the lambda function name
3. Add the permission: AmazonS3ObjectLambdaExecutionRolePolicy

## How to check this lambda

1. Go to S3
2. Click on Object Lambda Access Point
3. Click on your object lambda access point name
4. click on one uploaded PNG file
5. Click on the activated Open button

### Expected:
A new browser tab opens with a 128x128 thumbnail
