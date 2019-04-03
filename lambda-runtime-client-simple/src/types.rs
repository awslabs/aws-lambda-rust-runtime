use headers::{Header, HeaderMap, HeaderMapExt, HeaderName, HeaderValue};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

lazy_static! {
    static ref AWS_REQUEST_ID: HeaderName =
        HeaderName::from_static("lambda-runtime-aws-request-id");
    static ref AWS_INVOCATION_DEADLINE: HeaderName =
        HeaderName::from_static("lambda-runtime-deadline-ms");
    static ref AWS_FUNCTION_ARN: HeaderName =
        HeaderName::from_static("lambda-runtime-invoked-function-arn");
    static ref AWS_XRAY_TRACE_ID: HeaderName = HeaderName::from_static("lambda-runtime-trace-id");
    static ref AWS_MOBILE_CLIENT_CONTEXT: HeaderName =
        HeaderName::from_static("lambda-runtime-client-context");
    static ref AWS_MOBILE_CLIENT_IDENTITY: HeaderName =
        HeaderName::from_static("lambda-runtime-cognito-identity");
}

macro_rules! str_header {
    ($type:tt, $header_name:ident) => {
        impl Header for $type {
            fn name() -> &'static HeaderName {
                &$header_name
            }

            fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
            where
                I: Iterator<Item = &'i HeaderValue>,
            {
                let value = values
                    .next()
                    .and_then(|val| {
                        if let Ok(val) = val.to_str() {
                            return Some(String::from(val));
                        }
                        None
                    })
                    .ok_or_else(headers::Error::invalid)?;
                Ok($type(value))
            }

            fn encode<E>(&self, values: &mut E)
            where
                E: Extend<HeaderValue>,
            {
                let value = HeaderValue::from_str(&self.0).expect("Should not panic on encoding");
                values.extend(std::iter::once(value));
            }
        }
    };
}

macro_rules! num_header {
    ($type:tt, $header_name:ident) => {
        impl Header for $type {
            fn name() -> &'static HeaderName {
                &$header_name
            }

            fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
            where
                I: Iterator<Item = &'i HeaderValue>,
            {
                let value = values
                    .next()
                    .and_then(|val| {
                        if let Ok(val) = val.to_str() {
                            if let Ok(val) = val.parse::<u64>() {
                                return Some(val);
                            }
                        }
                        None
                    })
                    .ok_or_else(headers::Error::invalid)?;
                Ok($type(value))
            }

            fn encode<E>(&self, values: &mut E)
            where
                E: Extend<HeaderValue>,
            {
                let value = HeaderValue::from_str(&self.0.to_string())
                    .expect("Should not panic on encoding");
                values.extend(std::iter::once(value));
            }
        }
    };
}

/// The request ID, which identifies the request that triggered the function invocation. This header
/// tracks the invocation within the Lambda control plane. The request ID is used to specify completion
/// of a given invocation.
#[derive(Debug, Clone, PartialEq)]
pub struct RequestId(pub String);
str_header!(RequestId, AWS_REQUEST_ID);

/// The date that the function times out in Unix time milliseconds. For example, `1542409706888`.
#[derive(Debug, Clone, PartialEq)]
pub struct InvocationDeadline(pub u64);
num_header!(InvocationDeadline, AWS_INVOCATION_DEADLINE);

/// The ARN of the Lambda function, version, or alias that is specified in the invocation.
/// For instance, `arn:aws:lambda:us-east-2:123456789012:function:custom-runtime`.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionArn(pub String);
str_header!(FunctionArn, AWS_FUNCTION_ARN);

/// The AWS X-Ray Tracing header. For more information,
/// please see [AWS' documentation](https://docs.aws.amazon.com/xray/latest/devguide/xray-concepts.html#xray-concepts-tracingheader).
#[derive(Debug, Clone, PartialEq)]
pub struct XRayTraceId(pub String);
str_header!(XRayTraceId, AWS_XRAY_TRACE_ID);

/// For invocations from the AWS Mobile SDK contains data about client application and device.
#[derive(Debug, Clone, PartialEq)]
struct MobileClientContext(String);
str_header!(MobileClientContext, AWS_MOBILE_CLIENT_CONTEXT);

/// For invocations from the AWS Mobile SDK, data about the Amazon Cognito identity provider.
#[derive(Debug, Clone, PartialEq)]
struct MobileClientIdentity(String);
str_header!(MobileClientIdentity, AWS_MOBILE_CLIENT_IDENTITY);

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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Context {
    /// The AWS request ID generated by the Lambda service.
    pub aws_request_id: String,
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
}

impl Context {
    pub fn new(headers: HeaderMap<HeaderValue>) -> Option<Self> {
        let request_id = headers.typed_get::<RequestId>()?;
        let function_arn = headers.typed_get::<FunctionArn>()?;
        let deadline = headers.typed_get::<InvocationDeadline>()?;
        let xray = headers.typed_get::<XRayTraceId>();

        let ctx = Context {
            aws_request_id: request_id.0,
            deadline: deadline.0,
            invoked_function_arn: function_arn.0,
            xray_trace_id: xray.map(|v| v.0),
            ..Default::default()
        };
        Some(ctx)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{
        ClientApplication, ClientContext, CognitoIdentity, FunctionArn, InvocationDeadline,
        MobileClientContext, MobileClientIdentity, RequestId, XRayTraceId,
    };
    use bytes::Bytes;
    use headers::{HeaderMap, HeaderMapExt};
    use http::Response;
    use proptest::{collection, option, prelude::*, strategy::Strategy, string::string_regex};

    fn gen_request_id() -> impl Strategy<Value = RequestId> {
        let expr = "[0-9A-F]{8}-[0-9A-F]{4}-4[0-9A-F]{3}-[89AB][0-9A-F]{3}-[0-9A-F]{12}";
        let arbitrary_uuid = string_regex(expr).unwrap();
        arbitrary_uuid.prop_map(RequestId)
    }

    fn gen_invocation_deadline() -> impl Strategy<Value = InvocationDeadline> {
        any::<u64>().prop_map(InvocationDeadline)
    }

    fn gen_function_arn() -> impl Strategy<Value = FunctionArn> {
        let expr = "arn:aws:lambda:us-east-1:[0-9]{12}:function:custom-runtime";
        let arn = string_regex(expr).unwrap();
        arn.prop_map(FunctionArn)
    }

    fn gen_xray_trace_id() -> impl Strategy<Value = XRayTraceId> {
        let expr = "Root=1-[a-zA-Z0-9]{32};Parent=[a-z0-9]{16};Sampled=[0-1]{1}";
        let xray = string_regex(expr).unwrap();
        xray.prop_map(XRayTraceId)
    }

    fn uuid() -> impl Strategy<Value = String> {
        let expr = "[a-zA-Z0-9]{32}";
        string_regex(expr).unwrap()
    }

    fn gen_client_context() -> impl Strategy<Value = MobileClientContext> {
        uuid().prop_map(MobileClientContext)
    }

    fn gen_client_identity() -> impl Strategy<Value = MobileClientIdentity> {
        uuid().prop_map(MobileClientIdentity)
    }

    fn gen_client_identity_struct() -> impl Strategy<Value = CognitoIdentity> {
        (uuid()).prop_map(|uuid| CognitoIdentity {
            identity_id: uuid.clone(),
            identity_pool_id: uuid,
        })
    }

    fn gen_client_application() -> impl Strategy<Value = ClientApplication> {
        (uuid()).prop_map(|uuid| ClientApplication {
            installation_id: uuid.clone(),
            app_title: uuid.clone(),
            app_version_name: uuid.clone(),
            app_version_code: uuid.clone(),
            app_package_name: uuid,
        })
    }

    fn gen_client_context_struct() -> impl Strategy<Value = ClientContext> {
        let app = gen_client_application();
        let overrides = collection::hash_map(uuid(), uuid(), 1..10);
        let env = collection::hash_map(uuid(), uuid(), 1..10);
        (app, overrides, env).prop_map(|args| {
            let (app, overrides, env) = args;
            ClientContext {
                client: app,
                custom: overrides,
                environment: env,
            }
        })
    }

    fn gen_headers() -> impl Strategy<Value = HeaderMap> {
        let mandatory = (
            gen_request_id(),
            gen_invocation_deadline(),
            gen_function_arn(),
        );
        let xray = option::of(gen_xray_trace_id());
        let mobile = option::of((gen_client_context(), gen_client_identity()));
        (mandatory, xray, mobile).prop_map(|headers| {
            let (mandatory, xray, mobile) = headers;
            let mut map = HeaderMap::new();
            map.typed_insert(mandatory.0);
            map.typed_insert(mandatory.1);
            map.typed_insert(mandatory.2);
            xray.map(|xray| map.typed_insert(xray));
            mobile.map(|mobile| {
                map.typed_insert(mobile.0);
                map.typed_insert(mobile.1)
            });
            map
        })
    }

    fn gen_next_event() -> impl Strategy<Value = Response<Bytes>> {
        gen_headers().prop_map(|headers| {
            let mut resp = Response::new(Bytes::default());
            *resp.headers_mut() = headers;
            *resp.status_mut() = http::StatusCode::OK;
            resp
        })
    }

    #[test]
    fn request_id() {
        proptest!(|(req in gen_request_id())| {
            let mut headers = HeaderMap::new();
            headers.typed_insert(req.clone());
            prop_assert_eq!(headers.typed_get::<RequestId>(), Some(req));
        });
    }

    #[test]
    fn deadline() {
        proptest!(|(req in gen_invocation_deadline())| {
            let mut headers = HeaderMap::new();
            headers.typed_insert(req.clone());
            prop_assert_eq!(headers.typed_get::<InvocationDeadline>(), Some(req));
        });
    }

    #[test]
    fn function_arn() {
        proptest!(|(req in gen_function_arn())| {
            let mut headers = HeaderMap::new();
            headers.typed_insert(req.clone());
            prop_assert_eq!(headers.typed_get::<FunctionArn>(), Some(req));
        });
    }

    #[test]
    fn xray_trace_id() {
        proptest!(|(req in gen_xray_trace_id())| {
            let mut headers = HeaderMap::new();
            headers.typed_insert(req.clone());
            prop_assert_eq!(headers.typed_get::<XRayTraceId>(), Some(req));
        });
    }

    #[test]
    fn mobile_client_context() {
        proptest!(|(req in gen_client_context())| {
            let mut headers = HeaderMap::new();
            headers.typed_insert(req.clone());
            prop_assert_eq!(headers.typed_get::<MobileClientContext>(), Some(req));
        });
    }

    #[test]
    fn mobile_client_identity() {
        proptest!(|(req in gen_client_identity())| {
            let mut headers = HeaderMap::new();
            headers.typed_insert(req.clone());
            prop_assert_eq!(headers.typed_get::<MobileClientIdentity>(), Some(req));
        });
    }
}
