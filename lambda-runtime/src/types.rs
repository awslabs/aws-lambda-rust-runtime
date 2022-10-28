use crate::{Config, Error};
use http::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryFrom};

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Diagnostic<'a> {
    pub(crate) error_type: &'a str,
    pub(crate) error_message: &'a str,
}

/// The request ID, which identifies the request that triggered the function invocation. This header
/// tracks the invocation within the Lambda control plane. The request ID is used to specify completion
/// of a given invocation.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RequestId(pub String);

/// The date that the function times out in Unix time milliseconds. For example, `1542409706888`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct InvocationDeadline(pub u64);

/// The ARN of the Lambda function, version, or alias that is specified in the invocation.
/// For instance, `arn:aws:lambda:us-east-2:123456789012:function:custom-runtime`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FunctionArn(pub String);

/// The AWS X-Ray Tracing header. For more information,
/// please see [AWS' documentation](https://docs.aws.amazon.com/xray/latest/devguide/xray-concepts.html#xray-concepts-tracingheader).
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct XRayTraceId(pub String);

/// For invocations from the AWS Mobile SDK contains data about client application and device.
#[derive(Debug, Clone, Eq, PartialEq)]
struct MobileClientContext(String);

/// For invocations from the AWS Mobile SDK, data about the Amazon Cognito identity provider.
#[derive(Debug, Clone, Eq, PartialEq)]
struct MobileClientIdentity(String);

/// Client context sent by the AWS Mobile SDK.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ClientContext {
    /// Information about the mobile application invoking the function.
    #[serde(default)]
    pub client: ClientApplication,
    /// Custom properties attached to the mobile event context.
    #[serde(default)]
    pub custom: HashMap<String, String>,
    /// Environment settings from the mobile client.
    #[serde(default)]
    pub environment: HashMap<String, String>,
}

/// AWS Mobile SDK client fields.
#[derive(Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq)]
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
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
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
#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct Context {
    /// The AWS request ID generated by the Lambda service.
    pub request_id: String,
    /// The execution deadline for the current invocation in milliseconds.
    pub deadline: u64,
    /// The ARN of the Lambda function being invoked.
    pub invoked_function_arn: String,
    /// The X-Ray trace ID for the current invocation.
    pub xray_trace_id: Option<String>,
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
        let client_context: Option<ClientContext> = if let Some(value) = headers.get("lambda-runtime-client-context") {
            serde_json::from_str(value.to_str()?)?
        } else {
            None
        };

        let identity: Option<CognitoIdentity> = if let Some(value) = headers.get("lambda-runtime-cognito-identity") {
            serde_json::from_str(value.to_str()?)?
        } else {
            None
        };

        let ctx = Context {
            request_id: headers
                .get("lambda-runtime-aws-request-id")
                .expect("missing lambda-runtime-aws-request-id header")
                .to_str()?
                .to_owned(),
            deadline: headers
                .get("lambda-runtime-deadline-ms")
                .expect("missing lambda-runtime-deadline-ms header")
                .to_str()?
                .parse::<u64>()?,
            invoked_function_arn: headers
                .get("lambda-runtime-invoked-function-arn")
                .unwrap_or(&HeaderValue::from_static(
                    "No header lambda-runtime-invoked-function-arn found.",
                ))
                .to_str()?
                .to_owned(),
            xray_trace_id: headers
                .get("lambda-runtime-trace-id")
                .map(|v| String::from_utf8_lossy(v.as_bytes()).to_string()),
            client_context,
            identity,
            ..Default::default()
        };

        Ok(ctx)
    }
}

/// Incoming Lambda request containing the event payload and context.
#[derive(Clone, Debug)]
pub struct LambdaEvent<T> {
    /// Event payload.
    pub payload: T,
    /// Invocation context.
    pub context: Context,
}

impl<T> LambdaEvent<T> {
    /// Creates a new Lambda request
    pub fn new(payload: T, context: Context) -> Self {
        Self { payload, context }
    }

    /// Split the Lambda event into its payload and context.
    pub fn into_parts(self) -> (T, Context) {
        (self.payload, self.context)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn round_trip_lambda_error() {
        use serde_json::{json, Value};
        let expected = json!({
            "errorType": "InvalidEventDataError",
            "errorMessage": "Error parsing event data.",
        });

        let actual = Diagnostic {
            error_type: "InvalidEventDataError",
            error_message: "Error parsing event data.",
        };
        let actual: Value = serde_json::to_value(actual).expect("failed to serialize diagnostic");
        assert_eq!(expected, actual);
    }

    #[test]
    fn context_with_expected_values_and_types_resolves() {
        let mut headers = HeaderMap::new();
        headers.insert("lambda-runtime-aws-request-id", HeaderValue::from_static("my-id"));
        headers.insert("lambda-runtime-deadline-ms", HeaderValue::from_static("123"));
        headers.insert(
            "lambda-runtime-invoked-function-arn",
            HeaderValue::from_static("arn::myarn"),
        );
        headers.insert("lambda-runtime-trace-id", HeaderValue::from_static("arn::myarn"));
        let tried = Context::try_from(headers);
        assert!(tried.is_ok());
    }

    #[test]
    fn context_with_certain_missing_headers_still_resolves() {
        let mut headers = HeaderMap::new();
        headers.insert("lambda-runtime-aws-request-id", HeaderValue::from_static("my-id"));
        headers.insert("lambda-runtime-deadline-ms", HeaderValue::from_static("123"));
        let tried = Context::try_from(headers);
        assert!(tried.is_ok());
    }

    #[test]
    fn context_with_client_context_resolves() {
        let mut custom = HashMap::new();
        custom.insert("key".to_string(), "value".to_string());
        let mut environment = HashMap::new();
        environment.insert("key".to_string(), "value".to_string());
        let client_context = ClientContext {
            client: ClientApplication {
                installation_id: String::new(),
                app_title: String::new(),
                app_version_name: String::new(),
                app_version_code: String::new(),
                app_package_name: String::new(),
            },
            custom,
            environment,
        };
        let client_context_str = serde_json::to_string(&client_context).unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("lambda-runtime-aws-request-id", HeaderValue::from_static("my-id"));
        headers.insert("lambda-runtime-deadline-ms", HeaderValue::from_static("123"));
        headers.insert(
            "lambda-runtime-client-context",
            HeaderValue::from_str(&client_context_str).unwrap(),
        );
        let tried = Context::try_from(headers);
        assert!(tried.is_ok());
        let tried = tried.unwrap();
        assert!(tried.client_context.is_some());
        assert_eq!(tried.client_context.unwrap(), client_context);
    }

    #[test]
    fn context_with_empty_client_context_resolves() {
        let mut headers = HeaderMap::new();
        headers.insert("lambda-runtime-aws-request-id", HeaderValue::from_static("my-id"));
        headers.insert("lambda-runtime-deadline-ms", HeaderValue::from_static("123"));
        headers.insert("lambda-runtime-client-context", HeaderValue::from_static("{}"));
        let tried = Context::try_from(headers);
        assert!(tried.is_ok());
        assert!(tried.unwrap().client_context.is_some());
    }

    #[test]
    fn context_with_identity_resolves() {
        let cognito_identity = CognitoIdentity {
            identity_id: String::new(),
            identity_pool_id: String::new(),
        };
        let cognito_identity_str = serde_json::to_string(&cognito_identity).unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("lambda-runtime-aws-request-id", HeaderValue::from_static("my-id"));
        headers.insert("lambda-runtime-deadline-ms", HeaderValue::from_static("123"));
        headers.insert(
            "lambda-runtime-cognito-identity",
            HeaderValue::from_str(&cognito_identity_str).unwrap(),
        );
        let tried = Context::try_from(headers);
        assert!(tried.is_ok());
        let tried = tried.unwrap();
        assert!(tried.identity.is_some());
        assert_eq!(tried.identity.unwrap(), cognito_identity);
    }

    #[test]
    fn context_with_bad_deadline_type_is_err() {
        let mut headers = HeaderMap::new();
        headers.insert("lambda-runtime-aws-request-id", HeaderValue::from_static("my-id"));
        headers.insert(
            "lambda-runtime-deadline-ms",
            HeaderValue::from_static("BAD-Type,not <u64>"),
        );
        headers.insert(
            "lambda-runtime-invoked-function-arn",
            HeaderValue::from_static("arn::myarn"),
        );
        headers.insert("lambda-runtime-trace-id", HeaderValue::from_static("arn::myarn"));
        let tried = Context::try_from(headers);
        assert!(tried.is_err());
    }

    #[test]
    fn context_with_bad_client_context_is_err() {
        let mut headers = HeaderMap::new();
        headers.insert("lambda-runtime-aws-request-id", HeaderValue::from_static("my-id"));
        headers.insert("lambda-runtime-deadline-ms", HeaderValue::from_static("123"));
        headers.insert(
            "lambda-runtime-client-context",
            HeaderValue::from_static("BAD-Type,not JSON"),
        );
        let tried = Context::try_from(headers);
        assert!(tried.is_err());
    }

    #[test]
    fn context_with_empty_identity_is_err() {
        let mut headers = HeaderMap::new();
        headers.insert("lambda-runtime-aws-request-id", HeaderValue::from_static("my-id"));
        headers.insert("lambda-runtime-deadline-ms", HeaderValue::from_static("123"));
        headers.insert("lambda-runtime-cognito-identity", HeaderValue::from_static("{}"));
        let tried = Context::try_from(headers);
        assert!(tried.is_err());
    }

    #[test]
    fn context_with_bad_identity_is_err() {
        let mut headers = HeaderMap::new();
        headers.insert("lambda-runtime-aws-request-id", HeaderValue::from_static("my-id"));
        headers.insert("lambda-runtime-deadline-ms", HeaderValue::from_static("123"));
        headers.insert(
            "lambda-runtime-cognito-identity",
            HeaderValue::from_static("BAD-Type,not JSON"),
        );
        let tried = Context::try_from(headers);
        assert!(tried.is_err());
    }

    #[test]
    #[should_panic]
    #[allow(unused_must_use)]
    fn context_with_missing_request_id_should_panic() {
        let mut headers = HeaderMap::new();
        headers.insert("lambda-runtime-aws-request-id", HeaderValue::from_static("my-id"));
        headers.insert(
            "lambda-runtime-invoked-function-arn",
            HeaderValue::from_static("arn::myarn"),
        );
        headers.insert("lambda-runtime-trace-id", HeaderValue::from_static("arn::myarn"));
        Context::try_from(headers);
    }

    #[test]
    #[should_panic]
    #[allow(unused_must_use)]
    fn context_with_missing_deadline_should_panic() {
        let mut headers = HeaderMap::new();
        headers.insert("lambda-runtime-deadline-ms", HeaderValue::from_static("123"));
        headers.insert(
            "lambda-runtime-invoked-function-arn",
            HeaderValue::from_static("arn::myarn"),
        );
        headers.insert("lambda-runtime-trace-id", HeaderValue::from_static("arn::myarn"));
        Context::try_from(headers);
    }
}

impl Context {
    /// Add environment details to the context by setting `env_config`.
    pub fn with_config(self, config: &Config) -> Self {
        Self {
            env_config: config.clone(),
            ..self
        }
    }
}
