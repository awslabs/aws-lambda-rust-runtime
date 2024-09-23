# Rust Lambda with AppConfig Feature Flag

This project demonstrates a Rust-based AWS Lambda function that uses AWS AppConfig for feature flagging. The function is deployed using AWS CDK and includes automatic rollback capabilities based on error rates.

## Lambda Function (src/main.rs)

The Lambda function is written in Rust and does the following:

1. Integrates with AWS AppConfig to fetch configuration at runtime.
2. Uses a feature flag to determine whether to respond in Spanish.
3. Processes incoming events.
4. Returns a response based on the event and the current feature flag state.

The function is designed to work with the AWS AppConfig Extension for Lambda, allowing for efficient configuration retrieval.

## Deployment (cdk directory)

The project uses AWS CDK for infrastructure as code and deployment. To deploy the project:

1. Ensure you have the AWS CDK CLI installed and configured.
2. Navigate to the `cdk` directory.
3. Install dependencies:
   ```
   npm install
   ```
4. Build the CDK stack:
   ```
   npm run build
   ```
5. Deploy the stack:
   ```
   cdk deploy
   ```

## AWS Resources (cdk/lib/cdk-stack.ts)

The CDK stack creates the following AWS resources:

1. **AppConfig Application**: Named "MyRustLambdaApp", this is the container for your configuration and feature flags.

2. **AppConfig Environment**: A "Production" environment is created within the application.

3. **AppConfig Configuration Profile**: Defines the schema and validation for your configuration.

4. **AppConfig Hosted Configuration Version**: Contains the actual configuration data, including the "spanish-response" feature flag.

5. **AppConfig Deployment Strategy**: Defines how configuration changes are rolled out.

6. **Lambda Function**: A Rust-based function that uses the AppConfig configuration.
   - Uses the AWS AppConfig Extension Layer for efficient configuration retrieval.
   - Configured with ARM64 architecture and 128MB of memory.
   - 30-second timeout.

7. **CloudWatch Alarm**: Monitors the Lambda function's error rate.
   - Triggers if there are more than 5 errors per minute.

8. **AppConfig Deployment**: Connects all AppConfig components and includes a rollback trigger based on the CloudWatch alarm.

9. **IAM Role**: Grants the Lambda function permissions to interact with AppConfig and CloudWatch.

This setup allows for feature flagging with automatic rollback capabilities, ensuring rapid and safe deployment of new features or configurations.

## Usage

After deployment, you can update the feature flag in AppConfig to control the Lambda function's behavior. The function will automatically fetch the latest configuration, and if error rates exceed the threshold, AppConfig will automatically roll back to the previous stable configuration.
