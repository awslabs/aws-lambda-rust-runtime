use crate::{AWS_REQUEST_ID, FUNCTION_ARN, TRACE_ID};
use headers::{Error as HeaderError, Header, HeaderName, HeaderValue};

// TODO: this should be replaced with a proper derive, but the derive in hyperium/headers is private. PR soon!
macro_rules! impl_string_header {
    ($header_type:ty, $header_expr:expr, $header_name:expr) => {
        impl Header for $header_type {
            fn name() -> &'static HeaderName {
                &$header_name
            }

            fn decode<'i, I>(values: &mut I) -> Result<Self, HeaderError>
            where
                Self: Sized,
                I: Iterator<Item = &'i HeaderValue>,
            {
                let value: &HeaderValue = values.next().ok_or_else(headers::Error::invalid)?;
                let value: String = String::from(value.to_str().unwrap());
                Ok($header_expr(value))
            }

            fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
                let s = &self.0;
                let value = HeaderValue::from_str(&s).unwrap_or_else(|_| panic!("{} is not a valid HeaderValue", s));
                values.extend(std::iter::once(value))
            }
        }
    };
}

/// The request ID, which identifies the request that triggered the function invocation.
/// For example, `8476a536-e9f4-11e8-9739-2dfe598c3fcd`.
#[derive(Clone, Debug, PartialEq)]
pub struct AWSRequestId(pub String);
impl_string_header!(AWSRequestId, AWSRequestId, AWS_REQUEST_ID);

/// The ARN of the Lambda function, version, or alias that's specified in the invocation.
/// For example, `arn:aws:lambda:us-east-2:123456789012:function:custom-runtime`.
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionArn(pub String);
impl_string_header!(FunctionArn, FunctionArn, FUNCTION_ARN);

/// The [AWS X-Ray Tracing Header](https://docs.aws.amazon.com/xray/latest/devguide/xray-concepts.html#xray-concepts-tracingheader).
/// For example, `Root=1-5bef4de7-ad49b0e87f6ef6c87fc2e700;Parent=9a9197af755a6419;Sampled=1`.
#[derive(Clone, Debug, PartialEq)]
pub struct TraceId(pub String);
impl_string_header!(TraceId, TraceId, TRACE_ID);

#[cfg(test)]
mod typed_headers {
    use super::*;
    #[test]
    fn test_aws_request_id() {
        use headers::{HeaderMap, HeaderMapExt};

        let request_id = String::from("8476a536-e9f4-11e8-9739-2dfe598c3fcd");
        let request_id = AWSRequestId(request_id);
        let mut headers = HeaderMap::new();
        headers.typed_insert(request_id);
        assert!(headers.typed_get::<AWSRequestId>().is_some());
    }
}
