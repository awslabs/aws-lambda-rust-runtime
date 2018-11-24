use std::env;

use backtrace;
use env as lambda_env;
use error::HandlerError;
use lambda_runtime_client;

/// The Lambda function execution context. The values in this struct
/// are populated using the [Lambda environment variables](https://docs.aws.amazon.com/lambda/latest/dg/current-supported-versions.html)
/// and the headers returned by the poll request to the Runtime APIs.
/// A new instance of the `Context` object is passed to each handler invocation.
#[derive(Default, Clone)]
pub struct Context {
    /// The amount of memory allocated to the Lambda function in Mb.
    /// This value is extracted from the `AWS_LAMBDA_FUNCTION_MEMORY_SIZE`
    /// environment variable set by the Lambda service.
    pub memory_limit_in_mb: i32,
    /// The name of the Lambda function as registered with the Lambda
    /// service. The value is extracted from the `AWS_LAMBDA_FUNCTION_NAME`
    /// environment variable set by the Lambda service.
    pub function_name: String,
    /// The version of the function being invoked. This value is extracted
    /// from the `AWS_LAMBDA_FUNCTION_VERSION` environment variable set
    /// by the Lambda service.
    pub function_version: String,
    /// The fully qualified ARN (Amazon Resource Name) for the function
    /// invocation event. This value is returned by the Lambda Runtime APIs
    /// as a header.
    pub invoked_function_arn: String,
    /// The AWS request ID for the current invocation event. This value
    /// is returned by the Lambda Runtime APIs as a header.
    pub aws_request_id: String,
    /// The X-Ray trace ID for the current invocation. This value is returned
    /// by the Lambda Runtime APIs as a header. Developers can use this value
    /// with the AWS SDK to create new, custom sub-segments to the current
    /// invocation.
    pub xray_trace_id: String,
    /// The name of the CloudWatch log stream for the current execution
    /// environment. This value is extracted from the `AWS_LAMBDA_LOG_STREAM_NAME`
    /// environment variable set by the Lambda service.
    pub log_stream_name: String,
    /// The name of the CloudWatch log group for the current execution
    /// environment. This value is extracted from the `AWS_LAMBDA_LOG_GROUP_NAME`
    /// environment variable set by the Lambda service.
    pub log_group_name: String,

    /// The client context sent by the AWS Mobile SDK with the invocation
    /// request. This value is returned by the Lambda Runtime APIs as a
    /// header. This value is populated only if the invocation request
    /// originated from an AWS Mobile SDK or an SDK that attached the client
    /// context information to the request.
    pub client_context: Option<lambda_runtime_client::ClientContext>,
    /// The information of the Cognito identity that sent the invocation
    /// request to the Lambda service. This value is returned by the Lambda
    /// Runtime APIs in a header and it's only populated if the invocation
    /// request was performed with AWS credentials federated through the Cognito
    /// identity service.
    pub identity: Option<lambda_runtime_client::CognitoIdentity>,

    /// The deadline for the current handler execution in nanoseconds based
    /// on a unix `MONOTONIC` clock.
    pub(super) deadline: u128,
}

impl Context {
    /// Generates a new `Context` object for an event. Uses the responses headers alongside the
    /// environment variable values from the `FunctionSettings` object to populate the data.
    ///
    /// # Arguments
    ///
    /// * `local_settings` A populated environment settings object
    ///
    /// # Return
    /// A new, populated `Context` object.
    pub(super) fn new(local_settings: lambda_env::FunctionSettings) -> Context {
        Context {
            xray_trace_id: String::from(""),
            memory_limit_in_mb: local_settings.memory_size,
            function_name: local_settings.function_name,
            function_version: local_settings.version,
            log_stream_name: local_settings.log_stream,
            log_group_name: local_settings.log_group,
            ..Default::default()
        }
    }

    /// We use the context for each event to store the stack trace. This is the methods
    /// clients should use to retrieve an initialized `RuntimeError` with the populated
    /// stack trace.
    pub fn new_error(&self, msg: &str) -> HandlerError {
        let mut trace: Option<backtrace::Backtrace> = None;
        let is_backtrace = env::var("RUST_BACKTRACE");
        if is_backtrace.is_ok() && is_backtrace.unwrap() == "1" {
            trace!("Begin backtrace collection");
            trace = Option::from(backtrace::Backtrace::new());
            trace!("Completed backtrace collection");
        }
        HandlerError::new(msg, trace)
    }

    /// Returns the remaining time in the execution in milliseconds. This is based on the
    /// deadline header passed by Lambda's Runtime APIs.
    pub fn get_time_remaining_millis(&self) -> u128 {
        self.deadline - systime::get_time_ms()
    }
}

/// Iternal implementation used to retriev the current `MONOTINIC_CLOCK` time from
/// libc;

mod systime {
    use libc;

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub(super) fn get_time_ms() -> u128 {
        unsafe { u128::from(libc::mach_absolute_time()) / 1_000_000 }
    }

    #[cfg(not(any(windows, target_os = "macos", target_os = "ios")))]
    pub(super) fn get_time_ms() -> u128 {
        let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
        unsafe {
            libc::clock_gettime(libc::CLOCK_REALTIME, &mut ts);
        }
        return ((ts.tv_sec as u128) * 1_000) + ((ts.tv_nsec as u128) / 1_000_000);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use env::{self, ConfigProvider};
    use std::{thread::sleep, time};

    fn get_deadline(secs: u8) -> u128 {
        systime::get_time_ms() + (u128::from(secs) * 1_000)
    }

    pub(crate) fn test_context(deadline: u8) -> Context {
        Context {
            memory_limit_in_mb: 128,
            function_name: "test_func".to_string(),
            function_version: "$LATEST".to_string(),
            invoked_function_arn: "arn:aws:lambda".to_string(),
            aws_request_id: "123".to_string(),
            xray_trace_id: "123".to_string(),
            log_stream_name: "logStream".to_string(),
            log_group_name: "logGroup".to_string(),
            client_context: Option::default(),
            identity: Option::default(),
            deadline: get_deadline(deadline),
        }
    }

    #[test]
    fn verify_time_remaining() {
        let config = env::tests::MockConfigProvider { error: false };
        let mut ctx = Context::new(config.get_function_settings().unwrap());
        ctx.deadline = get_deadline(10);
        println!("Set deadline to: {}", ctx.deadline);
        sleep(time::Duration::new(2, 0));

        let remaining = ctx.get_time_remaining_millis();
        assert!(
            remaining > 7800 && remaining < 8200,
            "Remaining time in millis outside the expected range: {}",
            remaining
        );
    }
}
