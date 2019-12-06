use crate::error::{ApiError, ApiErrorKind, ErrorResponse};
use failure::ResultExt;
use hyper::{
    client::HttpConnector,
    header::{self, HeaderMap, HeaderValue},
    rt::{Future, Stream},
    Body, Client, Method, Request, Uri,
};
use log::*;
use serde_derive::*;
use serde_json;
use std::{collections::HashMap, fmt};
use tokio::runtime::Runtime;

const RUNTIME_API_VERSION: &str = "2018-06-01";
const API_CONTENT_TYPE: &str = "application/json";
const API_ERROR_CONTENT_TYPE: &str = "application/vnd.aws.lambda.error+json";
const RUNTIME_ERROR_HEADER: &str = "Lambda-Runtime-Function-Error-Type";
// TODO: Perhaps use a macro to generate this
const DEFAULT_AGENT: &str = "AWS_Lambda_Rust";

/// Enum of the headers returned by Lambda's `/next` API call.
pub enum LambdaHeaders {
    /// The AWS request ID
    RequestId,
    /// The ARN of the Lambda function being invoked
    FunctionArn,
    /// The X-Ray trace ID generated for this invocation
    TraceId,
    /// The deadline for the function execution in milliseconds
    Deadline,
    /// The client context header. This field is populated when the function
    /// is invoked from a mobile client.
    ClientContext,
    /// The Cognito Identity context for the invocation. This field is populated
    /// when the function is invoked with AWS credentials obtained from Cognito
    /// Identity.
    CognitoIdentity,
}

impl LambdaHeaders {
    /// Returns the `str` representation of the header.
    fn as_str(&self) -> &'static str {
        match self {
            LambdaHeaders::RequestId => "Lambda-Runtime-Aws-Request-Id",
            LambdaHeaders::FunctionArn => "Lambda-Runtime-Invoked-Function-Arn",
            LambdaHeaders::TraceId => "Lambda-Runtime-Trace-Id",
            LambdaHeaders::Deadline => "Lambda-Runtime-Deadline-Ms",
            LambdaHeaders::ClientContext => "Lambda-Runtime-Client-Context",
            LambdaHeaders::CognitoIdentity => "Lambda-Runtime-Cognito-Identity",
        }
    }
}

impl fmt::Display for LambdaHeaders {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// AWS Moble SDK client properties
#[derive(Deserialize, Clone)]
pub struct ClientApplication {
    /// The mobile app installation id
    #[serde(rename = "installationId")]
    pub installation_id: String,
    /// The app title for the mobile app as registered with AWS' mobile services.
    #[serde(rename = "appTitle")]
    pub app_title: String,
    /// The version name of the application as registered with AWS' mobile services.
    #[serde(rename = "appVersionName")]
    pub app_version_name: String,
    /// The app version code.
    #[serde(rename = "appVersionCode")]
    pub app_version_code: String,
    /// The package name for the mobile application invoking the function
    #[serde(rename = "appPackageName")]
    pub app_package_name: String,
}

/// Client context sent by the AWS Mobile SDK.
#[derive(Deserialize, Clone)]
pub struct ClientContext {
    /// Information about the mobile application invoking the function.
    pub client: ClientApplication,
    /// Custom properties attached to the mobile event context.
    pub custom: HashMap<String, String>,
    /// Environment settings from the mobile client.
    pub environment: HashMap<String, String>,
}

#[derive(Deserialize, Clone)]
/// Cognito identity information sent with the event
pub struct CognitoIdentity {
    /// The unique identity id for the Cognito credentials invoking the function.
    pub identity_id: String,
    /// The identity pool id the caller is "registered" with.
    pub identity_pool_id: String,
}

/// The Lambda function execution context. The values in this struct
/// are populated using the [Lambda environment variables](https://docs.aws.amazon.com/lambda/latest/dg/current-supported-versions.html)
/// and the headers returned by the poll request to the Runtime APIs.
/// A new instance of the `Context` object is passed to each handler invocation.
#[derive(Clone)]
pub struct EventContext {
    /// The ARN of the Lambda function being invoked.
    pub invoked_function_arn: String,
    /// The AWS request ID generated by the Lambda service.
    pub aws_request_id: String,
    /// The X-Ray trace ID for the current invocation.
    pub xray_trace_id: Option<String>,
    /// The execution deadline for the current invocation in milliseconds.
    pub deadline: i64,
    /// The client context object sent by the AWS mobile SDK. This field is
    /// empty unless the function is invoked using an AWS mobile SDK.
    pub client_context: Option<ClientContext>,
    /// The Cognito identity that invoked the function. This field is empty
    /// unless the invocation request to the Lambda APIs was made using AWS
    /// credentials issues by Amazon Cognito Identity Pools.
    pub identity: Option<CognitoIdentity>,
}

/// Used by the Runtime to communicate with the internal endpoint.
pub struct RuntimeClient {
    _runtime: Runtime,
    http_client: Client<HttpConnector, Body>,
    next_endpoint: Uri,
    runtime_agent: String,
    host: String,
}

impl<'ev> RuntimeClient {
    /// Creates a new instance of the Runtime APIclient SDK. The http client has timeouts disabled and
    /// will always send a `Connection: keep-alive` header. Optionally, the runtime client can receive
    /// a user agent string. This string is used to make requests to the runtime APIs and is used to
    /// identify the runtime being used by the function. For example, the `lambda_runtime_core` crate
    /// uses `AWS_Lambda_Rust/0.1.0 (rustc/1.31.1-stable)`. The runtime client can also receive an
    /// instance of Tokio Runtime to use.
    pub fn new(host: &str, agent: Option<String>, runtime: Option<Runtime>) -> Result<Self, ApiError> {
        debug!("Starting new HttpRuntimeClient for {}", host);
        let runtime_agent = match agent {
            Some(a) => a,
            None => DEFAULT_AGENT.to_owned(),
        };

        // start a tokio core main event loop for hyper
        let runtime = match runtime {
            Some(r) => r,
            None => Runtime::new().context(ApiErrorKind::Unrecoverable("Could not initialize runtime".to_string()))?,
        };

        let http_client = Client::builder().executor(runtime.executor()).build_http();
        // we cached the parsed Uri since this never changes.
        let next_endpoint = format!("http://{}/{}/runtime/invocation/next", host, RUNTIME_API_VERSION)
            .parse::<Uri>()
            .context(ApiErrorKind::Unrecoverable("Could not parse API uri".to_string()))?;

        Ok(RuntimeClient {
            _runtime: runtime,
            http_client,
            next_endpoint,
            runtime_agent,
            host: host.to_owned(),
        })
    }
}

impl<'ev> RuntimeClient {
    /// Polls for new events to the Runtime APIs.
    pub fn next_event(&self) -> Result<(Vec<u8>, EventContext), ApiError> {
        trace!("Polling for next event");

        let req = Request::builder()
            .method(Method::GET)
            .uri(self.next_endpoint.clone())
            .header(header::USER_AGENT, self.runtime_agent.clone())
            .body(Body::from(""))
            .unwrap();

        // We wait instead of processing the future asynchronously because AWS Lambda
        // itself enforces only one event per container at a time. No point in taking on
        // the additional complexity.
        let resp = self
            .http_client
            .request(req)
            .wait()
            .context(ApiErrorKind::Unrecoverable("Could not fetch next event".to_string()))?;

        if resp.status().is_client_error() {
            error!(
                "Runtime API returned client error when polling for new events: {}",
                resp.status()
            );
            Err(ApiErrorKind::Recoverable(format!(
                "Error {} when polling for events",
                resp.status()
            )))?;
        }
        if resp.status().is_server_error() {
            error!(
                "Runtime API returned server error when polling for new events: {}",
                resp.status()
            );
            Err(ApiErrorKind::Unrecoverable(
                "Server error when polling for new events".to_string(),
            ))?;
        }
        let ctx = self.get_event_context(&resp.headers())?;
        let out = resp
            .into_body()
            .concat2()
            .wait()
            .context(ApiErrorKind::Recoverable("Could not read event boxy".to_string()))?;
        let buf = out.into_bytes().to_vec();

        trace!(
            "Received new event for request id {}. Event length {} bytes",
            ctx.aws_request_id,
            buf.len()
        );
        Ok((buf, ctx))
    }

    /// Calls the Lambda Runtime APIs to submit a response to an event. In this function we treat
    /// all errors from the API as an unrecoverable error. This is because the API returns
    /// 4xx errors for responses that are too long. In that case, we simply log the output and fail.
    ///
    /// # Arguments
    ///
    /// * `request_id` The request id associated with the event we are serving the response for.
    ///                This is returned as a header from the poll (`/next`) API.
    /// * `output` The object be sent back to the Runtime APIs as a response.
    ///
    /// # Returns
    /// A `Result` object containing a bool return value for the call or an `error::ApiError` instance.
    pub fn event_response(&self, request_id: &str, output: &[u8]) -> Result<(), ApiError> {
        trace!(
            "Posting response for request {} to Runtime API. Response length {} bytes",
            request_id,
            output.len()
        );
        let uri = format!(
            "http://{}/{}/runtime/invocation/{}/response",
            self.host, RUNTIME_API_VERSION, request_id
        )
        .parse::<Uri>()
        .context(ApiErrorKind::Unrecoverable(
            "Could not generate response uri".to_owned(),
        ))?;
        let req = self.get_runtime_post_request(&uri, output);

        let resp = self
            .http_client
            .request(req)
            .wait()
            .context(ApiErrorKind::Recoverable("Could not post event response".to_string()))?;
        if !resp.status().is_success() {
            error!(
                "Error from Runtime API when posting response for request {}: {}",
                request_id,
                resp.status()
            );
            Err(ApiErrorKind::Recoverable(format!(
                "Error {} while sending response",
                resp.status()
            )))?;
        }
        trace!("Posted response to Runtime API for request {}", request_id);
        Ok(())
    }

    /// Calls Lambda's Runtime APIs to send an error generated by the `Handler`. Because it's rust,
    /// the error type for lambda is always `handled`.
    ///
    /// # Arguments
    ///
    /// * `request_id` The request id associated with the event we are serving the error for.
    /// * `e` An instance of `errors::HandlerError` generated by the handler function.
    ///
    /// # Returns
    /// A `Result` object containing a bool return value for the call or an `error::ApiError` instance.
    pub fn event_error(&self, request_id: &str, e: &ErrorResponse) -> Result<(), ApiError> {
        trace!(
            "Posting error to runtime API for request {}: {}",
            request_id,
            e.error_message
        );
        let uri = format!(
            "http://{}/{}/runtime/invocation/{}/error",
            self.host, RUNTIME_API_VERSION, request_id
        )
        .parse::<Uri>()
        .context(ApiErrorKind::Unrecoverable(
            "Could not generate response uri".to_owned(),
        ))?;
        let req = self.get_runtime_error_request(&uri, &e);

        let resp = self.http_client.request(req).wait().context(ApiErrorKind::Recoverable(
            "Could not post event error response".to_string(),
        ))?;
        if !resp.status().is_success() {
            error!(
                "Error from Runtime API when posting error response for request {}: {}",
                request_id,
                resp.status()
            );
            Err(ApiErrorKind::Recoverable(format!(
                "Error {} while sending response",
                resp.status()
            )))?;
        }
        trace!("Posted error response for request id {}", request_id);
        Ok(())
    }

    /// Calls the Runtime APIs to report a failure during the init process.
    /// The contents of the error (`e`) parmeter are passed to the Runtime APIs
    /// using the private `to_response()` method.
    ///
    /// # Arguments
    ///
    /// * `e` An instance of `errors::RuntimeError`.
    ///
    /// # Panics
    /// If it cannot send the init error. In this case we panic to force the runtime
    /// to restart.
    pub fn fail_init(&self, e: &ErrorResponse) {
        error!("Calling fail_init Runtime API: {}", e.error_message);
        let uri = format!("http://{}/{}/runtime/init/error", self.host, RUNTIME_API_VERSION)
            .parse::<Uri>()
            .map_err(|e| {
                error!("Could not parse fail init URI: {}", e);
                panic!("Killing runtime");
            });
        let req = self.get_runtime_error_request(&uri.unwrap(), &e);

        self.http_client
            .request(req)
            .wait()
            .map_err(|e| {
                error!("Error while sending init failed message: {}", e);
                panic!("Error while sending init failed message: {}", e);
            })
            .map(|resp| {
                info!("Successfully sent error response to the runtime API: {:?}", resp);
            })
            .expect("Could not complete init_fail request");
    }

    /// Returns the endpoint configured for this HTTP Runtime client.
    pub fn get_endpoint(&self) -> &str {
        &self.host
    }

    /// Creates a Hyper `Request` object for the given `Uri` and `Body`. Sets the
    /// HTTP method to `POST` and the `Content-Type` header value to `application/json`.
    ///
    /// # Arguments
    ///
    /// * `uri` A `Uri` reference target for the request
    /// * `body` The content of the post request. This parameter must not be null
    ///
    /// # Returns
    /// A Populated Hyper `Request` object.
    fn get_runtime_post_request(&self, uri: &Uri, body: &[u8]) -> Request<Body> {
        Request::builder()
            .method(Method::POST)
            .uri(uri.clone())
            .header(header::CONTENT_TYPE, header::HeaderValue::from_static(API_CONTENT_TYPE))
            .header(header::USER_AGENT, self.runtime_agent.clone())
            .body(Body::from(body.to_owned()))
            .unwrap()
    }

    fn get_runtime_error_request(&self, uri: &Uri, e: &ErrorResponse) -> Request<Body> {
        let body = serde_json::to_vec(&e).expect("Could not turn error object into response JSON");
        Request::builder()
            .method(Method::POST)
            .uri(uri.clone())
            .header(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static(API_ERROR_CONTENT_TYPE),
            )
            .header(header::USER_AGENT, self.runtime_agent.clone())
            // this header is static for the runtime APIs and it's likely to go away in the future.
            .header(RUNTIME_ERROR_HEADER, HeaderValue::from_static("Unhandled"))
            .body(Body::from(body))
            .unwrap()
    }

    /// Creates an `EventContext` object based on the response returned by the Runtime
    /// API `/next` endpoint.
    ///
    /// # Arguments
    ///
    /// * `resp` The response returned by the Runtime APIs endpoint.
    ///
    /// # Returns
    /// A `Result` containing the populated `EventContext` or an `ApiError` if the required headers
    /// were not present or the client context and cognito identity could not be parsed from the
    /// JSON string.
    fn get_event_context(&self, headers: &HeaderMap<HeaderValue>) -> Result<EventContext, ApiError> {
        // let headers = resp.headers();

        let aws_request_id = header_string(
            headers.get(LambdaHeaders::RequestId.as_str()),
            &LambdaHeaders::RequestId,
        )?;
        let invoked_function_arn = header_string(
            headers.get(LambdaHeaders::FunctionArn.as_str()),
            &LambdaHeaders::FunctionArn,
        )?;
        let xray_trace_id = match headers.get(LambdaHeaders::TraceId.as_str()) {
            Some(trace_id) => match trace_id.to_str() {
                Ok(trace_str) => Some(trace_str.to_owned()),
                Err(e) => {
                    // we do not fail on this error.
                    error!("Could not parse X-Ray trace id as string: {}", e);
                    None
                }
            },
            None => None,
        };
        let deadline = header_string(headers.get(LambdaHeaders::Deadline.as_str()), &LambdaHeaders::Deadline)?
            .parse::<i64>()
            .context(ApiErrorKind::Recoverable(
                "Could not parse deadline header value to int".to_string(),
            ))?;

        let mut ctx = EventContext {
            aws_request_id,
            invoked_function_arn,
            xray_trace_id,
            deadline,
            client_context: Option::default(),
            identity: Option::default(),
        };

        if let Some(ctx_json) = headers.get(LambdaHeaders::ClientContext.as_str()) {
            let ctx_json = ctx_json.to_str().context(ApiErrorKind::Recoverable(
                "Could not convert context header content to string".to_string(),
            ))?;
            trace!("Found Client Context in response headers: {}", ctx_json);
            let ctx_value: ClientContext = serde_json::from_str(&ctx_json).context(ApiErrorKind::Recoverable(
                "Could not parse client context value as json object".to_string(),
            ))?;
            ctx.client_context = Option::from(ctx_value);
        };

        if let Some(cognito_json) = headers.get(LambdaHeaders::CognitoIdentity.as_str()) {
            let cognito_json = cognito_json.to_str().context(ApiErrorKind::Recoverable(
                "Could not convert congnito context header content to string".to_string(),
            ))?;
            trace!("Found Cognito Identity in response headers: {}", cognito_json);
            let identity_value: CognitoIdentity = serde_json::from_str(&cognito_json).context(
                ApiErrorKind::Recoverable("Could not parse cognito context value as json object".to_string()),
            )?;
            ctx.identity = Option::from(identity_value);
        };

        Ok(ctx)
    }
}

fn header_string(value: Option<&HeaderValue>, header_type: &LambdaHeaders) -> Result<String, ApiError> {
    match value {
        Some(value_str) => Ok(value_str
            .to_str()
            .context(ApiErrorKind::Recoverable(format!(
                "Could not parse {} header",
                header_type
            )))?
            .to_owned()),
        None => {
            error!("Response headers do not contain {} header", header_type);
            Err(ApiErrorKind::Recoverable(format!("Missing {} header", header_type)))?
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    fn get_headers() -> HeaderMap<HeaderValue> {
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
        headers.insert(
            LambdaHeaders::RequestId.as_str(),
            HeaderValue::from_str("req_id").unwrap(),
        );
        headers.insert(
            LambdaHeaders::FunctionArn.as_str(),
            HeaderValue::from_str("func_arn").unwrap(),
        );
        headers.insert(LambdaHeaders::TraceId.as_str(), HeaderValue::from_str("trace").unwrap());
        let deadline = Utc::now() + Duration::seconds(10);
        headers.insert(
            LambdaHeaders::Deadline.as_str(),
            HeaderValue::from_str(&deadline.timestamp_millis().to_string()).unwrap(),
        );
        headers
    }

    #[test]
    fn get_event_context_with_empty_trace_id() {
        let client = RuntimeClient::new("localhost:8081", None, None).expect("Could not initialize runtime client");
        let mut headers = get_headers();
        headers.remove(LambdaHeaders::TraceId.as_str());
        let headers_result = client.get_event_context(&headers);
        assert_eq!(false, headers_result.is_err());
        let ok_result = headers_result.unwrap();
        assert_eq!(None, ok_result.xray_trace_id);
        assert_eq!("req_id", ok_result.aws_request_id);
    }

    #[test]
    fn get_event_context_populates_trace_id_when_present() {
        let client = RuntimeClient::new("localhost:8081", None, None).expect("Could not initialize runtime client");
        let headers = get_headers();
        let headers_result = client.get_event_context(&headers);
        assert_eq!(false, headers_result.is_err());
        assert_eq!(Some("trace".to_owned()), headers_result.unwrap().xray_trace_id);
    }
}
