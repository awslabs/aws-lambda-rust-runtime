use lambda_runtime::{service_fn, Diagnostic, Error, LambdaEvent};
use serde::Deserialize;

#[derive(Deserialize)]
struct Request {}

#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("transient database error: {0}")]
    DatabaseError(String),
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

impl From<ExecutionError> for Diagnostic {
    fn from(value: ExecutionError) -> Diagnostic {
        let (error_type, error_message) = match value {
            ExecutionError::DatabaseError(err) => ("Retryable", err.to_string()),
            ExecutionError::Unexpected(err) => ("NonRetryable", err.to_string()),
        };
        Diagnostic {
            error_type: error_type.into(),
            error_message,
        }
    }
}

/// This is the main body for the Lambda function
async fn function_handler(_event: LambdaEvent<Request>) -> Result<(), ExecutionError> {
    Err(ExecutionError::Unexpected("ooops".to_string()))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(service_fn(function_handler)).await
}
