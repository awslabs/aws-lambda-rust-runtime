/// AWS Lambda event definitions for activemq.
#[cfg(feature = "activemq")]
pub mod activemq;

/// AWS Lambda event definitions for alb.
#[cfg(feature = "alb")]
pub mod alb;
/// AWS Lambda event definitions for apigw.
#[cfg(feature = "apigw")]
pub mod apigw;

/// AWS Lambda event definitions for appsync.
#[cfg(feature = "appsync")]
pub mod appsync;

/// AWS Lambda event definitions for autoscaling.
#[cfg(feature = "autoscaling")]
pub mod autoscaling;

/// AWS Lambda event definitions for chime_bot.
#[cfg(feature = "chime_bot")]
pub mod chime_bot;

/// AWS Lambda event definitions for clientvpn.
#[cfg(feature = "clientvpn")]
pub mod clientvpn;

/// CloudWatch Events payload
#[cfg(feature = "cloudwatch_events")]
pub mod cloudwatch_events;

/// AWS Lambda event definitions for cloudwatch_logs.
#[cfg(feature = "cloudwatch_logs")]
pub mod cloudwatch_logs;

/// AWS Lambda event definitions for code_commit.
#[cfg(feature = "code_commit")]
pub mod code_commit;

/// AWS Lambda event definitions for codebuild.
#[cfg(feature = "codebuild")]
pub mod codebuild;

/// AWS Lambda event definitions for codedeploy.
#[cfg(feature = "codedeploy")]
pub mod codedeploy;

/// AWS Lambda event definitions for codepipeline_cloudwatch.
#[cfg(feature = "codepipeline_cloudwatch")]
pub mod codepipeline_cloudwatch;

/// AWS Lambda event definitions for codepipeline_job.
#[cfg(feature = "codepipeline_job")]
pub mod codepipeline_job;

/// AWS Lambda event definitions for cognito.
#[cfg(feature = "cognito")]
pub mod cognito;

/// AWS Lambda event definitions for config.
#[cfg(feature = "config")]
pub mod config;

/// AWS Lambda event definitions for connect.
#[cfg(feature = "connect")]
pub mod connect;

/// AWS Lambda event definitions for dynamodb.
#[cfg(feature = "dynamodb")]
pub mod dynamodb;

/// AWS Lambda event definitions for ecr_scan.
#[cfg(feature = "ecr_scan")]
pub mod ecr_scan;

/// AWS Lambda event definitions for firehose.
#[cfg(feature = "firehose")]
pub mod firehose;

/// AWS Lambda event definitions for iam.
#[cfg(feature = "iam")]
pub mod iam;

/// AWS Lambda event definitions for iot.
#[cfg(feature = "iot")]
pub mod iot;

/// AWS Lambda event definitions for iot_1_click.
#[cfg(feature = "iot_1_click")]
pub mod iot_1_click;

/// AWS Lambda event definitions for iot_button.
#[cfg(feature = "iot_button")]
pub mod iot_button;

/// AWS Lambda event definitions for iot_deprecated.
#[cfg(feature = "iot_deprecated")]
pub mod iot_deprecated;

/// AWS Lambda event definitions for kafka.
#[cfg(feature = "kafka")]
pub mod kafka;

/// AWS Lambda event definitions for kinesis.
#[cfg(feature = "kinesis")]
pub mod kinesis;

/// AWS Lambda event definitions for lambda_function_urls.
#[cfg(feature = "lambda_function_urls")]
pub mod lambda_function_urls;

/// AWS Lambda event definitions for lex.
#[cfg(feature = "lex")]
pub mod lex;

/// AWS Lambda event definitions for rabbitmq.
#[cfg(feature = "rabbitmq")]
pub mod rabbitmq;

/// AWS Lambda event definitions for s3.
#[cfg(feature = "s3")]
pub mod s3;

/// AWS Lambda event definitions for ses.
#[cfg(feature = "ses")]
pub mod ses;

/// AWS Lambda event definitions for SNS.
#[cfg(feature = "sns")]
pub mod sns;

/// AWS Lambda event definitions for SQS.
#[cfg(feature = "sqs")]
pub mod sqs;

/// AWS Lambda event definitions for streams.
#[cfg(feature = "streams")]
pub mod streams;
