use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::{boxed::Box, fmt, sync::Arc};
use tokio::sync::Mutex;
use tower::Service;
use tracing::{error, trace};

/// Payload received from the Telemetry API
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct LambdaTelemetry {
    /// Time when the telemetry was generated
    pub time: DateTime<Utc>,
    /// Telemetry record entry
    #[serde(flatten)]
    pub record: LambdaTelemetryRecord,
}

/// Record in a LambdaTelemetry entry
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "type", content = "record", rename_all = "lowercase")]
pub enum LambdaTelemetryRecord {
    /// Function log records
    Function(String),

    /// Extension log records
    Extension(String),

    /// Platform init start record
    #[serde(rename = "platform.initStart", rename_all = "camelCase")]
    PlatformInitStart {
        /// Type of initialization
        initialization_type: InitType,
        /// Phase of initialisation
        phase: InitPhase,
        /// Lambda runtime version
        runtime_version: Option<String>,
        /// Lambda runtime version ARN
        runtime_version_arn: Option<String>,
    },
    /// Platform init runtime done record
    #[serde(rename = "platform.initRuntimeDone", rename_all = "camelCase")]
    PlatformInitRuntimeDone {
        /// Type of initialization
        initialization_type: InitType,
        /// Phase of initialisation
        phase: Option<InitPhase>,
        /// Status of initalization
        status: Status,
        /// When the status = failure, the error_type describes what kind of error occurred
        error_type: Option<String>,
        /// Spans
        #[serde(default)]
        spans: Vec<Span>,
    },
    /// Platform init start record
    #[serde(rename = "platform.initReport", rename_all = "camelCase")]
    PlatformInitReport {
        /// Type of initialization
        initialization_type: InitType,
        /// Phase of initialisation
        phase: InitPhase,
        /// Metrics
        metrics: InitReportMetrics,
        /// Spans
        #[serde(default)]
        spans: Vec<Span>,
    },
    /// Record marking start of an invocation
    #[serde(rename = "platform.start", rename_all = "camelCase")]
    PlatformStart {
        /// Request identifier
        request_id: String,
        /// Version of the Lambda function
        version: Option<String>,
        /// Trace Context
        tracing: Option<TraceContext>,
    },
    /// Record marking the completion of an invocation
    #[serde(rename = "platform.runtimeDone", rename_all = "camelCase")]
    PlatformRuntimeDone {
        /// Request identifier
        request_id: String,
        /// Status of the invocation
        status: Status,
        /// When unsuccessful, the error_type describes what kind of error occurred
        error_type: Option<String>,
        /// Metrics corresponding to the runtime
        metrics: Option<RuntimeDoneMetrics>,
        /// Spans
        #[serde(default)]
        spans: Vec<Span>,
        /// Trace Context
        tracing: Option<TraceContext>,
    },
    /// Platfor report record
    #[serde(rename = "platform.report", rename_all = "camelCase")]
    PlatformReport {
        /// Request identifier
        request_id: String,
        /// Status of the invocation
        status: Status,
        /// When unsuccessful, the error_type describes what kind of error occurred
        error_type: Option<String>,
        /// Metrics
        metrics: ReportMetrics,
        /// Spans
        #[serde(default)]
        spans: Vec<Span>,
        /// Trace Context
        tracing: Option<TraceContext>,
    },

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
    /// Telemetry processor-specific record
    #[serde(rename = "platform.telemetrySubscription", rename_all = "camelCase")]
    PlatformTelemetrySubscription {
        /// Name of the extension
        name: String,
        /// State of the extensions
        state: String,
        /// Types of records sent to the extension
        types: Vec<String>,
    },
    /// Record generated when the telemetry processor is falling behind
    #[serde(rename = "platform.logsDropped", rename_all = "camelCase")]
    PlatformLogsDropped {
        /// Reason for dropping the logs
        reason: String,
        /// Number of records dropped
        dropped_records: u64,
        /// Total size of the dropped records
        dropped_bytes: u64,
    },
}

/// Type of Initialization
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum InitType {
    /// Initialised on demand
    OnDemand,
    /// Initialized to meet the provisioned concurrency
    ProvisionedConcurrency,
    /// SnapStart
    SnapStart,
}

/// Phase in which initialization occurs
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum InitPhase {
    /// Initialization phase
    Init,
    /// Invocation phase
    Invoke,
}

/// Status of invocation/initialization
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    /// Success
    Success,
    /// Error
    Error,
    /// Failure
    Failure,
    /// Timeout
    Timeout,
}

/// Span
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Span {
    /// Duration of the span
    pub duration_ms: f64,
    /// Name of the span
    pub name: String,
    /// Start of the span
    pub start: DateTime<Utc>,
}

/// Tracing Context
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TraceContext {
    /// Span ID
    pub span_id: Option<String>,
    /// Type of tracing
    pub r#type: TracingType,
    /// A string containing tracing information like trace_id. The contents may depend on the TracingType.
    pub value: String,
}

/// Type of tracing
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub enum TracingType {
    /// Amazon trace type
    #[serde(rename = "X-Amzn-Trace-Id")]
    AmznTraceId,
}

///Init report metrics
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InitReportMetrics {
    /// Duration of initialization
    pub duration_ms: f64,
}

/// Report metrics
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReportMetrics {
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
    /// Restore duration in milliseconds
    #[serde(default = "Option::default")]
    pub restore_duration_ms: Option<f64>,
}

/// Runtime done metrics
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDoneMetrics {
    /// Duration in milliseconds
    pub duration_ms: f64,
    /// Number of bytes produced as a result of the invocation
    pub produced_bytes: Option<u64>,
}

/// Wrapper function that sends telemetry to the subscriber Service
///
/// This takes an `hyper::Request` and transforms it into `Vec<LambdaTelemetry>` for the
/// underlying `Service` to process.
pub(crate) async fn telemetry_wrapper<S>(
    service: Arc<Mutex<S>>,
    req: hyper::Request<hyper::Body>,
) -> Result<hyper::Response<hyper::Body>, Box<dyn std::error::Error + Send + Sync>>
where
    S: Service<Vec<LambdaTelemetry>, Response = ()>,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + fmt::Debug,
    S::Future: Send,
{
    trace!("Received telemetry request");
    // Parse the request body as a Vec<LambdaTelemetry>
    let body = match hyper::body::to_bytes(req.into_body()).await {
        Ok(body) => body,
        Err(e) => {
            error!("Error reading telemetry request body: {}", e);
            return Ok(hyper::Response::builder()
                .status(hyper::StatusCode::BAD_REQUEST)
                .body(hyper::Body::empty())
                .unwrap());
        }
    };

    let telemetry: Vec<LambdaTelemetry> = match serde_json::from_slice(&body) {
        Ok(telemetry) => telemetry,
        Err(e) => {
            error!("Error parsing telemetry: {}", e);
            return Ok(hyper::Response::builder()
                .status(hyper::StatusCode::BAD_REQUEST)
                .body(hyper::Body::empty())
                .unwrap());
        }
    };

    {
        let mut service = service.lock().await;
        match service.call(telemetry).await {
            Ok(_) => (),
            Err(err) => println!("{:?}", err),
        }
    }

    Ok(hyper::Response::new(hyper::Body::empty()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    macro_rules! deserialize_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input, expected) = $value;
                    let actual = serde_json::from_str::<LambdaTelemetry>(&input).expect("unable to deserialize");

                    assert!(actual.record == expected);
                }
            )*
        }
    }

    deserialize_tests! {
        // function
        function: (
            r#"{"time": "2020-08-20T12:31:32.123Z","type": "function", "record": "hello world"}"#,
            LambdaTelemetryRecord::Function("hello world".to_string()),
        ),

        // extension
        extension: (
            r#"{"time": "2020-08-20T12:31:32.123Z","type": "extension", "record": "hello world"}"#,
            LambdaTelemetryRecord::Extension("hello world".to_string()),
        ),

        // platform.start
        platform_start: (
            r#"{"time":"2022-10-21T14:05:03.165Z","type":"platform.start","record":{"requestId":"459921b5-681c-4a96-beb0-81e0aa586026","version":"$LATEST","tracing":{"spanId":"24cd7d670fa455f0","type":"X-Amzn-Trace-Id","value":"Root=1-6352a70e-1e2c502e358361800241fd45;Parent=35465b3a9e2f7c6a;Sampled=1"}}}"#,
            LambdaTelemetryRecord::PlatformStart {
                request_id: "459921b5-681c-4a96-beb0-81e0aa586026".to_string(),
                version: Some("$LATEST".to_string()),
                tracing: Some(TraceContext{
                    span_id: Some("24cd7d670fa455f0".to_string()),
                    r#type: TracingType::AmznTraceId,
                    value: "Root=1-6352a70e-1e2c502e358361800241fd45;Parent=35465b3a9e2f7c6a;Sampled=1".to_string(),
                }),
            },
        ),
        // platform.initStart
        platform_init_start: (
            r#"{"time":"2022-10-19T13:52:15.636Z","type":"platform.initStart","record":{"initializationType":"on-demand","phase":"init"}}"#,
            LambdaTelemetryRecord::PlatformInitStart {
                initialization_type: InitType::OnDemand,
                phase: InitPhase::Init,
                runtime_version: None,
                runtime_version_arn: None,
            },
        ),
        // platform.runtimeDone
        platform_runtime_done: (
            r#"{"time":"2022-10-21T14:05:05.764Z","type":"platform.runtimeDone","record":{"requestId":"459921b5-681c-4a96-beb0-81e0aa586026","status":"success","tracing":{"spanId":"24cd7d670fa455f0","type":"X-Amzn-Trace-Id","value":"Root=1-6352a70e-1e2c502e358361800241fd45;Parent=35465b3a9e2f7c6a;Sampled=1"},"spans":[{"name":"responseLatency","start":"2022-10-21T14:05:03.165Z","durationMs":2598.0},{"name":"responseDuration","start":"2022-10-21T14:05:05.763Z","durationMs":0.0}],"metrics":{"durationMs":2599.0,"producedBytes":8}}}"#,
            LambdaTelemetryRecord::PlatformRuntimeDone {
                request_id: "459921b5-681c-4a96-beb0-81e0aa586026".to_string(),
                status: Status::Success,
                error_type: None,
                metrics: Some(RuntimeDoneMetrics {
                    duration_ms: 2599.0,
                    produced_bytes: Some(8),
                }),
                spans: vec!(
                    Span {
                        name:"responseLatency".to_string(),
                        start: Utc.ymd(2022, 10, 21).and_hms_milli(14, 05, 03, 165),
                        duration_ms:2598.0
                    },
                    Span {
                        name:"responseDuration".to_string(),
                        start:Utc.ymd(2022, 10, 21).and_hms_milli(14, 05, 05, 763),
                        duration_ms:0.0
                    },
                ),
                tracing: Some(TraceContext{
                    span_id: Some("24cd7d670fa455f0".to_string()),
                    r#type: TracingType::AmznTraceId,
                    value: "Root=1-6352a70e-1e2c502e358361800241fd45;Parent=35465b3a9e2f7c6a;Sampled=1".to_string(),
                }),
            },
        ),
        // platform.report
        platform_report: (
            r#"{"time":"2022-10-21T14:05:05.766Z","type":"platform.report","record":{"requestId":"459921b5-681c-4a96-beb0-81e0aa586026","metrics":{"durationMs":2599.4,"billedDurationMs":2600,"memorySizeMB":128,"maxMemoryUsedMB":94,"initDurationMs":549.04},"tracing":{"spanId":"24cd7d670fa455f0","type":"X-Amzn-Trace-Id","value":"Root=1-6352a70e-1e2c502e358361800241fd45;Parent=35465b3a9e2f7c6a;Sampled=1"},"status":"success"}}"#,
            LambdaTelemetryRecord::PlatformReport {
                request_id: "459921b5-681c-4a96-beb0-81e0aa586026".to_string(),
                status: Status::Success,
                error_type: None,
                metrics: ReportMetrics {
                    duration_ms: 2599.4,
                    billed_duration_ms: 2600,
                    memory_size_mb:128,
                    max_memory_used_mb:94,
                    init_duration_ms: Some(549.04),
                    restore_duration_ms: None,
                },
                spans: Vec::new(),
                tracing: Some(TraceContext {
                    span_id: Some("24cd7d670fa455f0".to_string()),
                    r#type: TracingType::AmznTraceId,
                    value: "Root=1-6352a70e-1e2c502e358361800241fd45;Parent=35465b3a9e2f7c6a;Sampled=1".to_string(),
                }),
            },
        ),
        // platform.telemetrySubscription
        platform_telemetry_subscription: (
            r#"{"time":"2022-10-19T13:52:15.667Z","type":"platform.telemetrySubscription","record":{"name":"my-extension","state":"Subscribed","types":["platform","function"]}}"#,
            LambdaTelemetryRecord::PlatformTelemetrySubscription {
                 name: "my-extension".to_string(),
                 state: "Subscribed".to_string(),
                 types: vec!("platform".to_string(), "function".to_string()),
            },
        ),
        // platform.initRuntimeDone
        platform_init_runtime_done: (
            r#"{"time":"2022-10-19T13:52:16.136Z","type":"platform.initRuntimeDone","record":{"initializationType":"on-demand","status":"success"}}"#,
            LambdaTelemetryRecord::PlatformInitRuntimeDone {
                initialization_type: InitType::OnDemand,
                status: Status::Success,
                phase: None,
                error_type: None,
                spans: Vec::new(),
            },
        ),
        // platform.extension
        platform_extension: (
            r#"{"time":"2022-10-19T13:52:16.136Z","type":"platform.extension","record":{"name":"my-extension","state":"Ready","events":["SHUTDOWN","INVOKE"]}}"#,
            LambdaTelemetryRecord::PlatformExtension {
                name: "my-extension".to_string(),
                state: "Ready".to_string(),
                events: vec!("SHUTDOWN".to_string(), "INVOKE".to_string()),
             },
        ),
        // platform.initReport
        platform_init_report: (
            r#"{"time":"2022-10-19T13:52:16.136Z","type":"platform.initReport","record":{"initializationType":"on-demand","metrics":{"durationMs":500.0},"phase":"init"}}"#,
            LambdaTelemetryRecord::PlatformInitReport {
                initialization_type: InitType::OnDemand,
                phase: InitPhase::Init,
                metrics: InitReportMetrics { duration_ms: 500.0 },
                spans: Vec::new(),
            }
        ),
    }
}
