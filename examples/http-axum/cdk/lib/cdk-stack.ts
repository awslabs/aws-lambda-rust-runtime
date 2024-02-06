import { join } from 'path';
import { CfnOutput, Stack, StackProps } from 'aws-cdk-lib';
import { Construct } from 'constructs';
import { RustFunction } from 'cargo-lambda-cdk';
import { LambdaRestApi } from 'aws-cdk-lib/aws-apigateway'
import { FunctionUrlAuthType } from "aws-cdk-lib/aws-lambda";

export class CdkStack extends Stack {
  constructor(scope: Construct, id: string, props?: StackProps) {
    super(scope, id, props);

    const handler = new RustFunction(this, 'Axum API', {
      // Path to the http-axum root directory.
      manifestPath: join(__dirname, '..', '..'),
    });

    if (process.env.ENABLE_LAMBDA_RUST_AXUM_FUNCTION_URL) {
      const lambdaUrl = handler.addFunctionUrl({
        authType: FunctionUrlAuthType.NONE,
      });
      new CfnOutput(this, 'Axum FunctionUrl ', { value: lambdaUrl.url });
    }

    new LambdaRestApi(this, 'axum', { handler });
  }
}
