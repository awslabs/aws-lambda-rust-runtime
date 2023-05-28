#![deny(rust_2018_idioms)]
#[cfg(feature = "http")]
pub use http;
#[cfg(feature = "query_map")]
pub use query_map;

mod custom_serde;
/// Encodings used in AWS Lambda json event values.
pub mod encodings;
#[cfg(feature = "chrono")]
pub mod time_window;

/// AWS Lambda event definitions.
pub mod event;

/// AWS Lambda event definitions for activemq.
#[cfg(feature = "activemq")]
pub use event::activemq;

/// AWS Lambda event definitions for alb.
#[cfg(feature = "alb")]
pub use event::alb;
/// AWS Lambda event definitions for apigw.
#[cfg(feature = "apigw")]
pub use event::apigw;

/// AWS Lambda event definitions for appsync.
#[cfg(feature = "appsync")]
pub use event::appsync;

/// AWS Lambda event definitions for autoscaling.
#[cfg(feature = "autoscaling")]
pub use event::autoscaling;

/// AWS Lambda event definitions for chime_bot.
#[cfg(feature = "chime_bot")]
pub use event::chime_bot;

/// AWS Lambda event definitions for clientvpn.
#[cfg(feature = "clientvpn")]
pub use event::clientvpn;

/// CloudWatch Events payload
#[cfg(feature = "cloudwatch_events")]
pub use event::cloudwatch_events;

/// AWS Lambda event definitions for cloudwatch_logs.
#[cfg(feature = "cloudwatch_logs")]
pub use event::cloudwatch_logs;

/// AWS Lambda event definitions for code_commit.
#[cfg(feature = "code_commit")]
pub use event::code_commit;

/// AWS Lambda event definitions for codebuild.
#[cfg(feature = "codebuild")]
pub use event::codebuild;

/// AWS Lambda event definitions for codedeploy.
#[cfg(feature = "codedeploy")]
pub use event::codedeploy;

/// AWS Lambda event definitions for codepipeline_cloudwatch.
#[cfg(feature = "codepipeline_cloudwatch")]
pub use event::codepipeline_cloudwatch;

/// AWS Lambda event definitions for codepipeline_job.
#[cfg(feature = "codepipeline_job")]
pub use event::codepipeline_job;

/// AWS Lambda event definitions for cognito.
#[cfg(feature = "cognito")]
pub use event::cognito;

/// AWS Lambda event definitions for config.
#[cfg(feature = "config")]
pub use event::config;

/// AWS Lambda event definitions for connect.
#[cfg(feature = "connect")]
pub use event::connect;

/// AWS Lambda event definitions for dynamodb.
#[cfg(feature = "dynamodb")]
pub use event::dynamodb;

/// AWS Lambda event definitions for ecr_scan.
#[cfg(feature = "ecr_scan")]
pub use event::ecr_scan;

/// AWS Lambda event definitions for firehose.
#[cfg(feature = "firehose")]
pub use event::firehose;

/// AWS Lambda event definitions for iam.
#[cfg(feature = "iam")]
pub use event::iam;

/// AWS Lambda event definitions for iot.
#[cfg(feature = "iot")]
pub use event::iot;

/// AWS Lambda event definitions for iot_1_click.
#[cfg(feature = "iot_1_click")]
pub use event::iot_1_click;

/// AWS Lambda event definitions for iot_button.
#[cfg(feature = "iot_button")]
pub use event::iot_button;

/// AWS Lambda event definitions for iot_deprecated.
#[cfg(feature = "iot_deprecated")]
pub use event::iot_deprecated;

/// AWS Lambda event definitions for kafka.
#[cfg(feature = "kafka")]
pub use event::kafka;

/// AWS Lambda event definitions for kinesis.
#[cfg(feature = "kinesis")]
pub use event::kinesis;

/// AWS Lambda event definitions for kinesis_analytics.
#[cfg(feature = "kinesis_analytics")]
pub use event::kinesis::analytics as kinesis_analytics;

/// AWS Lambda event definitions for lambda_function_urls.
#[cfg(feature = "lambda_function_urls")]
pub use event::lambda_function_urls;

/// AWS Lambda event definitions for lex.
#[cfg(feature = "lex")]
pub use event::lex;

/// AWS Lambda event definitions for rabbitmq.
#[cfg(feature = "rabbitmq")]
pub use event::rabbitmq;

/// AWS Lambda event definitions for s3.
#[cfg(feature = "s3")]
pub use event::s3;

/// AWS Lambda event definitions for s3_batch_job.
#[cfg(feature = "s3")]
pub use event::s3::batch_job as s3_batch_job;

/// AWS Lambda event definitions for ses.
#[cfg(feature = "ses")]
pub use event::ses;

/// AWS Lambda event definitions for SNS.
#[cfg(feature = "sns")]
pub use event::sns;

/// AWS Lambda event definitions for SQS.
#[cfg(feature = "sqs")]
pub use event::sqs;

/// AWS Lambda event definitions for streams.
#[cfg(feature = "streams")]
pub use event::streams;
