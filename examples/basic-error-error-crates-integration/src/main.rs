use lambda_runtime::{run, service_fn, Diagnostic, IntoDiagnostic, Error, LambdaEvent};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum ErrorType {
    Anyhow,
    Eyre,
    Miette,
}

#[derive(Deserialize)]
struct Request {
    error_type: ErrorType,
}

fn anyhow_error() -> anyhow::Result<()> {
    anyhow::bail!("This is an error message from Anyhow");
}

fn eyre_error() -> eyre::Result<()> {
    eyre::bail!("This is an error message from Eyre");
}

fn miette_error() -> miette::Result<()> {
    miette::bail!("This is an error message from Miette");
}

/// Transform an anyhow::Error, eyre::Report, or miette::Report into a lambda_runtime::Diagnostic.
/// It does it by enabling the feature `anyhow`, `eyre` or `miette` in the runtime dependency,
/// and importing the `IntoDiagnostic` trait, which enables
/// the implementation of `into_diagnostic` for `anyhow::Error`, `eyre::Report`, and `miette::Report`.
async fn function_handler(event: LambdaEvent<Request>) -> Result<(), Diagnostic> {
    match event.payload.error_type {
        ErrorType::Anyhow => anyhow_error().map_err(|e| e.into_diagnostic()),
        ErrorType::Eyre => eyre_error().map_err(|e| e.into_diagnostic()),
        ErrorType::Miette => miette_error().map_err(|e| e.into_diagnostic()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(function_handler)).await
}
