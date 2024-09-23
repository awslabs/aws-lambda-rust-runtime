import * as cdk from 'aws-cdk-lib';
import * as appconfig from 'aws-cdk-lib/aws-appconfig';
import * as lambda from 'aws-cdk-lib/aws-lambda';
import * as cloudwatch from 'aws-cdk-lib/aws-cloudwatch';
import { Construct } from 'constructs';
import { RustFunction } from 'cargo-lambda-cdk';

export class CdkStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    // Create AppConfig Application
    const application = new appconfig.CfnApplication(this, 'MyApplication', {
      name: 'MyRustLambdaApp',
    });

    // Create AppConfig Environment
    const environment = new appconfig.CfnEnvironment(this, 'MyEnvironment', {
      applicationId: application.ref,
      name: 'Production',
    });

    // Create AppConfig Configuration Profile
    const configProfile = new appconfig.CfnConfigurationProfile(this, 'MyConfigProfile', {
      applicationId: application.ref,
      name: 'MyConfigProfile',
      locationUri: 'hosted',
    });

    // Create AppConfig Hosted Configuration Version
    const hostedConfig = new appconfig.CfnHostedConfigurationVersion(this, 'MyHostedConfig', {
      applicationId: application.ref,
      configurationProfileId: configProfile.ref,
      content: JSON.stringify({
        'spanish-response': false
      }),
      contentType: 'application/json',
    });

    // Create AppConfig Deployment Strategy
    const deploymentStrategy = new appconfig.CfnDeploymentStrategy(this, 'MyDeploymentStrategy', {
      name: 'MyDeploymentStrategy',
      deploymentDurationInMinutes: 0,
      growthFactor: 100,
      replicateTo: 'NONE',
    });

    const architecture = lambda.Architecture.ARM_64;
    const layerVersion = architecture === lambda.Architecture.ARM_64 ? '68' : '60';

    // Create Lambda function using cargo-lambda-cdk
    const myFunction = new RustFunction(this, 'MyRustFunction', {
      functionName: 'my-rust-lambda',
      manifestPath: '..', // Points to the parent directory where Cargo.toml is located
      architecture,
      memorySize: 128,
      timeout: cdk.Duration.seconds(30),
      environment: {
        APPLICATION_ID: application.ref,
        ENVIRONMENT_ID: environment.ref,
        CONFIGURATION_PROFILE_ID: configProfile.ref,
        AWS_APPCONFIG_EXTENSION_PREFETCH_LIST: `/applications/${application.ref}/environments/${environment.ref}/configurations/${configProfile.ref}`,
      },
      layers: [
        lambda.LayerVersion.fromLayerVersionArn(
          this,
          'AppConfigExtensionLayer',
          `arn:aws:lambda:${this.region}:027255383542:layer:AWS-AppConfig-Extension:${layerVersion}`
        ),
      ],
    });

    // Create CloudWatch Alarm for rollback
    const errorRateAlarm = new cloudwatch.Alarm(this, 'ErrorRateAlarm', {
      metric: myFunction.metricErrors({
        period: cdk.Duration.minutes(1),
        statistic: 'sum',
      }),
      threshold: 5,
      evaluationPeriods: 1,
      comparisonOperator: cloudwatch.ComparisonOperator.GREATER_THAN_THRESHOLD,
      alarmDescription: 'Alarm if the error rate is greater than 5 errors per minute',
    });

    // Create AppConfig Deployment with rollback configuration
    new appconfig.CfnDeployment(this, 'MyDeployment', {
      applicationId: application.ref,
      environmentId: environment.ref,
      deploymentStrategyId: deploymentStrategy.ref,
      configurationProfileId: configProfile.ref,
      configurationVersion: hostedConfig.ref,
      tags: [
        {
          key: 'RollbackTrigger',
          value: errorRateAlarm.alarmArn,
        },
      ],
    });

    // Grant AppConfig permissions to the Lambda function
    myFunction.addToRolePolicy(new cdk.aws_iam.PolicyStatement({
      actions: [
        'appconfig:GetConfiguration',
        'appconfig:StartConfigurationSession',
        'cloudwatch:PutMetricData',
      ],
      resources: ['*'],
    }));
  }
}
