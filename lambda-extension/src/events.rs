use serde::Deserialize;

/// Request tracing information
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tracing {
    /// The type of tracing exposed to the extension
    pub r#type: String,
    /// The span value
    pub value: String,
}
/// Event received when there is a new Lambda invocation.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvokeEvent {
    /// The time that the function times out
    pub deadline_ms: u64,
    /// The ID assigned to the Lambda request
    pub request_id: String,
    /// The function's Amazon Resource Name
    pub invoked_function_arn: String,
    /// The request tracing information
    pub tracing: Tracing,
}

/// Event received when a Lambda function shuts down.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShutdownEvent {
    /// The reason why the function terminates
    /// It can be SPINDOWN, TIMEOUT, or FAILURE
    pub shutdown_reason: String,
    /// The time that the function times out
    pub deadline_ms: u64,
}

/// Event that the extension receives in
/// either the INVOKE or SHUTDOWN phase
#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE", tag = "eventType")]
pub enum NextEvent {
    /// Payload when the event happens in the INVOKE phase
    Invoke(InvokeEvent),
    /// Payload when the event happens in the SHUTDOWN phase
    Shutdown(ShutdownEvent),
}

impl NextEvent {
    /// Return whether the event is a [`NextEvent::Invoke`] event or not
    pub fn is_invoke(&self) -> bool {
        matches!(self, NextEvent::Invoke(_))
    }
}

/// Wrapper with information about the next
/// event that the Lambda Runtime is going to process
pub struct LambdaEvent {
    /// ID assigned to this extension by the Lambda Runtime
    pub extension_id: String,
    /// Next incoming event
    pub next: NextEvent,
}

impl LambdaEvent {
    pub(crate) fn new(ex_id: &str, next: NextEvent) -> LambdaEvent {
        LambdaEvent {
            extension_id: ex_id.into(),
            next,
        }
    }
}
