use chrono::{DateTime, Utc};
use http::{Request, Response};
use http_body_util::BodyExt;
use hyper::body::Incoming;
use lambda_runtime_api_client::body::Body;
use serde::{Deserialize, Serialize};
use std::{fmt, sync::Arc};
use tokio::sync::Mutex;
use tower::Service;
use tracing::{error, trace};

use crate::{Error, ExtensionError};

/// Payload received from the Lambda Logs API
/// See: https://docs.aws.amazon.com/lambda/latest/dg/runtimes-logs-api.html#runtimes-logs-api-msg
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct LambdaLog {
    /// Time when the log was generated
    pub time: DateTime<Utc>,
    /// Log record entry
    #[serde(flatten)]
    pub record: LambdaLogRecord,
}

/// Record in a LambdaLog entry
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "type", content = "record", rename_all = "lowercase")]
pub enum LambdaLogRecord {
    /// Function log records
    Function(String),

    /// Extension log records
    Extension(String),

    /// Platform start record
    #[serde(rename = "platform.start", rename_all = "camelCase")]
    PlatformStart {
        /// Request identifier
        request_id: String,
    },
    /// Platform stop record
    #[serde(rename = "platform.end", rename_all = "camelCase")]
    PlatformEnd {
        /// Request identifier
        request_id: String,
    },
    /// Platform report record
    #[serde(rename = "platform.report", rename_all = "camelCase")]
    PlatformReport {
        /// Request identifier
        request_id: String,
        /// Request metrics
        metrics: LogPlatformReportMetrics,
    },
    /// Runtime or execution environment error record
    #[serde(rename = "platform.fault")]
    PlatformFault(String),
    /// Extension-specific record
    #[serde(rename = "platform.extension", rename_all = "camelCase")]
    PlatformExtension {
        /// Name of the extension
        name: String,
        /// State of the extension
        state: String,
        /// Events sent to the extension
        events: Vec<String>,
    },
    /// Log processor-specific record
    #[serde(rename = "platform.logsSubscription", rename_all = "camelCase")]
    PlatformLogsSubscription {
        /// Name of the extension
        name: String,
        /// State of the extensions
        state: String,
        /// Types of records sent to the extension
        types: Vec<String>,
    },
    /// Record generated when the log processor is falling behind
    #[serde(rename = "platform.logsDropped", rename_all = "camelCase")]
    PlatformLogsDropped {
        /// Reason for dropping the logs
        reason: String,
        /// Number of records dropped
        dropped_records: u64,
        /// Total size of the dropped records
        dropped_bytes: u64,
    },
    /// Record marking the completion of an invocation
    #[serde(rename = "platform.runtimeDone", rename_all = "camelCase")]
    PlatformRuntimeDone {
        /// Request identifier
        request_id: String,
        /// Status of the invocation
        status: String,
    },
}

/// Platform report metrics
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LogPlatformReportMetrics {
    /// Duration in milliseconds
    pub duration_ms: f64,
    /// Billed duration in milliseconds
    pub billed_duration_ms: u64,
    /// Memory allocated in megabytes
    #[serde(rename = "memorySizeMB")]
    pub memory_size_mb: u64,
    /// Maximum memory used for the invoke in megabytes
    #[serde(rename = "maxMemoryUsedMB")]
    pub max_memory_used_mb: u64,
    /// Init duration in case of a cold start
    #[serde(default = "Option::default")]
    pub init_duration_ms: Option<f64>,
}

/// Log buffering configuration.
/// Allows Lambda to buffer logs before delivering them to a subscriber.
#[derive(Debug, Serialize, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LogBuffering {
    /// The maximum time (in milliseconds) to buffer a batch.
    /// Default: 1,000. Minimum: 25. Maximum: 30,000
    pub timeout_ms: usize,
    /// The maximum size (in bytes) of the logs to buffer in memory.
    /// Default: 262,144. Minimum: 262,144. Maximum: 1,048,576
    pub max_bytes: usize,
    /// The maximum number of events to buffer in memory.
    /// Default: 10,000. Minimum: 1,000. Maximum: 10,000
    pub max_items: usize,
}

static LOG_BUFFERING_MIN_TIMEOUT_MS: usize = 25;
static LOG_BUFFERING_MAX_TIMEOUT_MS: usize = 30_000;
static LOG_BUFFERING_MIN_BYTES: usize = 262_144;
static LOG_BUFFERING_MAX_BYTES: usize = 1_048_576;
static LOG_BUFFERING_MIN_ITEMS: usize = 1_000;
static LOG_BUFFERING_MAX_ITEMS: usize = 10_000;

impl LogBuffering {
    fn validate(&self) -> Result<(), Error> {
        if self.timeout_ms < LOG_BUFFERING_MIN_TIMEOUT_MS || self.timeout_ms > LOG_BUFFERING_MAX_TIMEOUT_MS {
            let error = format!(
                "LogBuffering validation error: Invalid timeout_ms: {}. Allowed values: Minumun: {}. Maximum: {}",
                self.timeout_ms, LOG_BUFFERING_MIN_TIMEOUT_MS, LOG_BUFFERING_MAX_TIMEOUT_MS
            );
            return Err(ExtensionError::boxed(error));
        }
        if self.max_bytes < LOG_BUFFERING_MIN_BYTES || self.max_bytes > LOG_BUFFERING_MAX_BYTES {
            let error = format!(
                "LogBuffering validation error: Invalid max_bytes: {}. Allowed values: Minumun: {}. Maximum: {}",
                self.max_bytes, LOG_BUFFERING_MIN_BYTES, LOG_BUFFERING_MAX_BYTES
            );
            return Err(ExtensionError::boxed(error));
        }
        if self.max_items < LOG_BUFFERING_MIN_ITEMS || self.max_items > LOG_BUFFERING_MAX_ITEMS {
            let error = format!(
                "LogBuffering validation error: Invalid max_items: {}. Allowed values: Minumun: {}. Maximum: {}",
                self.max_items, LOG_BUFFERING_MIN_ITEMS, LOG_BUFFERING_MAX_ITEMS
            );
            return Err(ExtensionError::boxed(error));
        }
        Ok(())
    }
}

impl Default for LogBuffering {
    fn default() -> Self {
        LogBuffering {
            timeout_ms: 1_000,
            max_bytes: 262_144,
            max_items: 10_000,
        }
    }
}

/// Validate the `LogBuffering` configuration (if present)
///
/// # Errors
///
/// This function will return an error if `LogBuffering` is present and configured incorrectly
pub(crate) fn validate_buffering_configuration(log_buffering: Option<LogBuffering>) -> Result<(), Error> {
    match log_buffering {
        Some(log_buffering) => log_buffering.validate(),
        None => Ok(()),
    }
}

/// Wrapper function that sends logs to the subscriber Service
///
/// This takes an `hyper::Request` and transforms it into `Vec<LambdaLog>` for the
/// underlying `Service` to process.
pub(crate) async fn log_wrapper<S>(service: Arc<Mutex<S>>, req: Request<Incoming>) -> Result<Response<Body>, Error>
where
    S: Service<Vec<LambdaLog>, Response = ()>,
    S::Error: Into<Error> + fmt::Debug,
    S::Future: Send,
{
    trace!("Received logs request");
    // Parse the request body as a Vec<LambdaLog>
    let body = match req.into_body().collect().await {
        Ok(body) => body,
        Err(e) => {
            error!("Error reading logs request body: {}", e);
            return Ok(hyper::Response::builder()
                .status(hyper::StatusCode::BAD_REQUEST)
                .body(Body::empty())
                .unwrap());
        }
    };
    let logs: Vec<LambdaLog> = match serde_json::from_slice(&body.to_bytes()) {
        Ok(logs) => logs,
        Err(e) => {
            error!("Error parsing logs: {}", e);
            return Ok(hyper::Response::builder()
                .status(hyper::StatusCode::BAD_REQUEST)
                .body(Body::empty())
                .unwrap());
        }
    };

    {
        let mut service = service.lock().await;
        match service.call(logs).await {
            Ok(_) => (),
            Err(err) => println!("{err:?}"),
        }
    }

    Ok(hyper::Response::new(Body::empty()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};

    #[test]
    fn deserialize_full() {
        let data = r#"{"time": "2020-08-20T12:31:32.123Z","type": "function", "record": "hello world"}"#;
        let expected = LambdaLog {
            time: Utc
                .with_ymd_and_hms(2020, 8, 20, 12, 31, 32)
                .unwrap()
                .checked_add_signed(Duration::milliseconds(123))
                .unwrap(),
            record: LambdaLogRecord::Function("hello world".to_string()),
        };

        let actual = serde_json::from_str::<LambdaLog>(data).unwrap();

        assert_eq!(expected, actual);
    }

    macro_rules! deserialize_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input, expected) = $value;
                    let actual = serde_json::from_str::<LambdaLog>(&input).expect("unable to deserialize");

                    assert!(actual.record == expected);
                }
            )*
        }
    }

    deserialize_tests! {
        // function
        function: (
            r#"{"time": "2020-08-20T12:31:32.123Z","type": "function", "record": "hello world"}"#,
            LambdaLogRecord::Function("hello world".to_string()),
        ),

        // extension
        extension: (
            r#"{"time": "2020-08-20T12:31:32.123Z","type": "extension", "record": "hello world"}"#,
            LambdaLogRecord::Extension("hello world".to_string()),
        ),

        // platform.start
        platform_start: (
            r#"{"time": "2020-08-20T12:31:32.123Z","type": "platform.start","record": {"requestId": "6f7f0961f83442118a7af6fe80b88d56"}}"#,
            LambdaLogRecord::PlatformStart {
                request_id: "6f7f0961f83442118a7af6fe80b88d56".to_string(),
            },
        ),
        // platform.end
        platform_end: (
            r#"{"time": "2020-08-20T12:31:32.123Z","type": "platform.end","record": {"requestId": "6f7f0961f83442118a7af6fe80b88d56"}}"#,
            LambdaLogRecord::PlatformEnd {
                request_id: "6f7f0961f83442118a7af6fe80b88d56".to_string(),
            },
        ),
        // platform.report
        platform_report: (
            r#"{"time": "2020-08-20T12:31:32.123Z","type": "platform.report","record": {"requestId": "6f7f0961f83442118a7af6fe80b88d56","metrics": {"durationMs": 1.23,"billedDurationMs": 123,"memorySizeMB": 123,"maxMemoryUsedMB": 123,"initDurationMs": 1.23}}}"#,
            LambdaLogRecord::PlatformReport {
                request_id: "6f7f0961f83442118a7af6fe80b88d56".to_string(),
                metrics: LogPlatformReportMetrics {
                    duration_ms: 1.23,
                    billed_duration_ms: 123,
                    memory_size_mb: 123,
                    max_memory_used_mb: 123,
                    init_duration_ms: Some(1.23),
                },
            },
        ),
        // platform.fault
        platform_fault: (
            r#"{"time": "2020-08-20T12:31:32.123Z","type": "platform.fault","record": "RequestId: d783b35e-a91d-4251-af17-035953428a2c Process exited before completing request"}"#,
            LambdaLogRecord::PlatformFault(
                "RequestId: d783b35e-a91d-4251-af17-035953428a2c Process exited before completing request"
                    .to_string(),
            ),
        ),
        // platform.extension
        platform_extension: (
            r#"{"time": "2020-08-20T12:31:32.123Z","type": "platform.extension","record": {"name": "Foo.bar","state": "Ready","events": ["INVOKE", "SHUTDOWN"]}}"#,
            LambdaLogRecord::PlatformExtension {
                name: "Foo.bar".to_string(),
                state: "Ready".to_string(),
                events: vec!["INVOKE".to_string(), "SHUTDOWN".to_string()],
            },
        ),
        // platform.logsSubscription
        platform_logssubscription: (
            r#"{"time": "2020-08-20T12:31:32.123Z","type": "platform.logsSubscription","record": {"name": "test","state": "active","types": ["test"]}}"#,
            LambdaLogRecord::PlatformLogsSubscription {
                name: "test".to_string(),
                state: "active".to_string(),
                types: vec!["test".to_string()],
            },
        ),
        // platform.logsDropped
        platform_logsdropped: (
            r#"{"time": "2020-08-20T12:31:32.123Z","type": "platform.logsDropped","record": {"reason": "Consumer seems to have fallen behind as it has not acknowledged receipt of logs.","droppedRecords": 123,"droppedBytes": 12345}}"#,
            LambdaLogRecord::PlatformLogsDropped {
                reason: "Consumer seems to have fallen behind as it has not acknowledged receipt of logs."
                    .to_string(),
                dropped_records: 123,
                dropped_bytes: 12345,
            },
        ),
        // platform.runtimeDone
        platform_runtimedone: (
            r#"{"time": "2021-02-04T20:00:05.123Z","type": "platform.runtimeDone","record": {"requestId":"6f7f0961f83442118a7af6fe80b88d56","status": "success"}}"#,
            LambdaLogRecord::PlatformRuntimeDone {
                request_id: "6f7f0961f83442118a7af6fe80b88d56".to_string(),
                status: "success".to_string(),
            },
        ),
    }

    macro_rules! log_buffering_configuration_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input, expected) = $value;
                    let result = validate_buffering_configuration(input);

                    if let Some(expected) = expected {
                        assert!(result.is_err());
                        assert_eq!(result.unwrap_err().to_string(), expected.to_string());
                    } else {
                        assert!(result.is_ok());
                    }

                }
            )*
        }
    }

    log_buffering_configuration_tests! {
        log_buffer_configuration_none_success: (
            None,
            None::<ExtensionError>
        ),
        log_buffer_configuration_default_success: (
            Some(LogBuffering::default()),
            None::<ExtensionError>
        ),
        log_buffer_configuration_min_success: (
            Some(LogBuffering { timeout_ms: LOG_BUFFERING_MIN_TIMEOUT_MS, max_bytes: LOG_BUFFERING_MIN_BYTES, max_items: LOG_BUFFERING_MIN_ITEMS }),
            None::<ExtensionError>
        ),
        log_buffer_configuration_max_success: (
            Some(LogBuffering { timeout_ms: LOG_BUFFERING_MAX_TIMEOUT_MS, max_bytes: LOG_BUFFERING_MAX_BYTES, max_items: LOG_BUFFERING_MAX_ITEMS }),
            None::<ExtensionError>
        ),
        min_timeout_ms_error: (
            Some(LogBuffering { timeout_ms: LOG_BUFFERING_MIN_TIMEOUT_MS-1, max_bytes: LOG_BUFFERING_MAX_BYTES, max_items: LOG_BUFFERING_MAX_ITEMS }),
            Some(ExtensionError::boxed("LogBuffering validation error: Invalid timeout_ms: 24. Allowed values: Minumun: 25. Maximum: 30000"))
        ),
        max_timeout_ms_error: (
            Some(LogBuffering { timeout_ms: LOG_BUFFERING_MAX_TIMEOUT_MS+1, max_bytes: LOG_BUFFERING_MAX_BYTES, max_items: LOG_BUFFERING_MAX_ITEMS }),
            Some(ExtensionError::boxed("LogBuffering validation error: Invalid timeout_ms: 30001. Allowed values: Minumun: 25. Maximum: 30000"))
        ),
        min_max_bytes_error: (
            Some(LogBuffering { timeout_ms: LOG_BUFFERING_MAX_TIMEOUT_MS, max_bytes: LOG_BUFFERING_MIN_BYTES-1, max_items: LOG_BUFFERING_MAX_ITEMS }),
            Some(ExtensionError::boxed("LogBuffering validation error: Invalid max_bytes: 262143. Allowed values: Minumun: 262144. Maximum: 1048576"))
        ),
        max_max_bytes_error: (
            Some(LogBuffering { timeout_ms: LOG_BUFFERING_MAX_TIMEOUT_MS, max_bytes: LOG_BUFFERING_MAX_BYTES+1, max_items: LOG_BUFFERING_MAX_ITEMS }),
            Some(ExtensionError::boxed("LogBuffering validation error: Invalid max_bytes: 1048577. Allowed values: Minumun: 262144. Maximum: 1048576"))
        ),
        min_max_items_error: (
            Some(LogBuffering { timeout_ms: LOG_BUFFERING_MAX_TIMEOUT_MS, max_bytes: LOG_BUFFERING_MAX_BYTES, max_items: LOG_BUFFERING_MIN_ITEMS-1 }),
            Some(ExtensionError::boxed("LogBuffering validation error: Invalid max_items: 999. Allowed values: Minumun: 1000. Maximum: 10000"))
        ),
        max_max_items_error: (
            Some(LogBuffering { timeout_ms: LOG_BUFFERING_MAX_TIMEOUT_MS, max_bytes: LOG_BUFFERING_MAX_BYTES, max_items: LOG_BUFFERING_MAX_ITEMS+1 }),
            Some(ExtensionError::boxed("LogBuffering validation error: Invalid max_items: 10001. Allowed values: Minumun: 1000. Maximum: 10000"))
        ),
    }
}
