use crate::{Config, Error};
use http::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryFrom};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Diagnostic {
    pub(crate) error_type: String,
    pub(crate) error_message: String,
}

#[test]
fn round_trip_lambda_error() -> Result<(), Error> {
    use serde_json::{json, Value};
    let expected = json!({
        "errorType": "InvalidEventDataError",
        "errorMessage": "Error parsing event data.",
    });

    let actual: Diagnostic = serde_json::from_value(expected.clone())?;
    let actual: Value = serde_json::to_value(actual)?;
    assert_eq!(expected, actual);

    Ok(())
}

/// The request ID, which identifies the request that triggered the function invocation. This header
/// tracks the invocation within the Lambda control plane. The request ID is used to specify completion
/// of a given invocation.
#[derive(Debug, Clone, PartialEq)]
pub struct RequestId(pub String);

/// The date that the function times out in Unix time milliseconds. For example, `1542409706888`.
#[derive(Debug, Clone, PartialEq)]
pub struct InvocationDeadline(pub u64);

/// The ARN of the Lambda function, version, or alias that is specified in the invocation.
/// For instance, `arn:aws:lambda:us-east-2:123456789012:function:custom-runtime`.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionArn(pub String);

/// The AWS X-Ray Tracing header. For more information,
/// please see [AWS' documentation](https://docs.aws.amazon.com/xray/latest/devguide/xray-concepts.html#xray-concepts-tracingheader).
#[derive(Debug, Clone, PartialEq)]
pub struct XRayTraceId(pub String);

/// For invocations from the AWS Mobile SDK contains data about client application and device.
#[derive(Debug, Clone, PartialEq)]
struct MobileClientContext(String);

/// For invocations from the AWS Mobile SDK, data about the Amazon Cognito identity provider.
#[derive(Debug, Clone, PartialEq)]
struct MobileClientIdentity(String);

/// Client context sent by the AWS Mobile SDK.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClientContext {
    /// Information about the mobile application invoking the function.
    pub client: ClientApplication,
    /// Custom properties attached to the mobile event context.
    pub custom: HashMap<String, String>,
    /// Environment settings from the mobile client.
    pub environment: HashMap<String, String>,
}

/// AWS Mobile SDK client fields.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClientApplication {
    /// The mobile app installation id
    pub installation_id: String,
    /// The app title for the mobile app as registered with AWS' mobile services.
    pub app_title: String,
    /// The version name of the application as registered with AWS' mobile services.
    pub app_version_name: String,
    /// The app version code.
    pub app_version_code: String,
    /// The package name for the mobile application invoking the function
    pub app_package_name: String,
}

/// Cognito identity information sent with the event
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CognitoIdentity {
    /// The unique identity id for the Cognito credentials invoking the function.
    pub identity_id: String,
    /// The identity pool id the caller is "registered" with.
    pub identity_pool_id: String,
}

/// The Lambda function execution context. The values in this struct
/// are populated using the [Lambda environment variables](https://docs.aws.amazon.com/lambda/latest/dg/current-supported-versions.html)
/// and the headers returned by the poll request to the Runtime APIs.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Context {
    /// The AWS request ID generated by the Lambda service.
    pub request_id: String,
    /// The execution deadline for the current invocation in milliseconds.
    pub deadline: u64,
    /// The ARN of the Lambda function being invoked.
    pub invoked_function_arn: String,
    /// The X-Ray trace ID for the current invocation.
    pub xray_trace_id: String,
    /// The client context object sent by the AWS mobile SDK. This field is
    /// empty unless the function is invoked using an AWS mobile SDK.
    pub client_context: Option<ClientContext>,
    /// The Cognito identity that invoked the function. This field is empty
    /// unless the invocation request to the Lambda APIs was made using AWS
    /// credentials issues by Amazon Cognito Identity Pools.
    pub identity: Option<CognitoIdentity>,
    /// Lambda function configuration from the local environment variables.
    /// Includes information such as the function name, memory allocation,
    /// version, and log streams.
    pub env_config: Config,
}

impl TryFrom<HeaderMap> for Context {
    type Error = Error;
    fn try_from(headers: HeaderMap) -> Result<Self, Self::Error> {
        let ctx = Context {
            request_id: headers.get("lambda-runtime-aws-request-id")
                .unwrap_or(&HeaderValue::from_str("").unwrap())
                .to_str()
                .expect("Missing Request ID")
                .to_owned(),
            deadline: headers.get("lambda-runtime-deadline-ms")
                .unwrap_or(&HeaderValue::from_str("100").unwrap())
                .to_str()?
                .parse()
                .expect("Missing deadline"),
            invoked_function_arn: headers.get("lambda-runtime-invoked-function-arn")
                .unwrap_or(&HeaderValue::from_str("").unwrap())
                .to_str()
                .expect("Missing arn; this is a bug")
                .to_owned(),
            xray_trace_id: headers.get("lambda-runtime-trace-id")
                .unwrap_or(&HeaderValue::from_str("").unwrap())
                .to_str()
                .expect("Invalid XRayTraceID sent by Lambda; this is a bug")
                .to_owned(),
            ..Default::default()
        };
        Ok(ctx)
    }
}
