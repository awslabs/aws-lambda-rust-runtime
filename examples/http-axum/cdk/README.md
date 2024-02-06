# Axum HTTP CDK Stack

This is a basic stack that shows how to deploy the Axum HTTP example with the AWS CDK.

## Resources

This stack deploys the Axum HTTP example in AWS Lambda.

It also creates an API Gateway Rest API to expose the Axum app to the internet. When the deploy is completed, the stack will print the endpoint URL for the gateway. It will look something like this:

```
CdkStack.axumEndpointC1B330D3 = https://sr0e4dqg1b.execute-api.us-east-1.amazonaws.com/prod/
```

If you set the environment variable `ENABLE_LAMBDA_RUST_AXUM_FUNCTION_URL=true` in your terminal before deploying the stack, it will also create a Lambda Function URL without any authentication mode. When the deploy completes, the stack will print the endpoint URL for this function. It will look something like this:

```
CdkStack.AxumFunctionUrl = https://7st53uq3rpk4jweki2ek765gty0icvuf.lambda-url.us-east-1.on.aws/
```

## Dependencies

1. Install the AWS CDK with NPM: `npm install -g cdk`.
2. Install [cargo-lambda](https://github.com/cargo-lambda/cargo-lambda#installation)

## Deployment

Run `npm run build` to complile the stack.

Then, run `npm run cdk deploy` to deploy the stack on your AWS account.

## Security

This example doesn't provide any security configuration. It's up to you to configure the stack with the security settings that are more convenient to you. We're not responsible for resources open to the internet on your AWS account.

## Cleanup

Deploying this stack on your account might incur on AWS costs due to the resources that we're deploying. Don't forget to delete those resources from your account if you're not using them any longer.

Run `npm run cdk destroy` to delete all resources in this stack from your AWS account.
