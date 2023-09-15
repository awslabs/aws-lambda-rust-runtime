# AWS Lambda Function example

## Build & Deploy

1. Install [cargo-lambda](https://github.com/cargo-lambda/cargo-lambda#installation)
2. Build the function with `cargo lambda build --release`
3. Deploy the function to AWS Lambda with `cargo lambda deploy --enable-function-url --iam-role YOUR_ROLE`
4. Enable Lambda streaming response on Lambda console: change the function url's invoke mode to `RESPONSE_STREAM`
5. Verify the function works: `curl -v -N <function-url>`. The results should be streamed back with 0.5 second pause between each word.

## Build for ARM 64

Build the function with `cargo lambda build --release --arm64`
