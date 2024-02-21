use axum::{
    async_trait,
    extract::{FromRequest, Request},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use lambda_http::{run, Error, RequestExt};
use serde_json::{json, Value};
use std::{collections::HashMap, env::set_var};

struct AuthorizerField(String);
struct AuthorizerFields(HashMap<String, serde_json::Value>);

#[async_trait]
impl<S> FromRequest<S> for AuthorizerField
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        req.request_context_ref()
            .and_then(|r| r.authorizer())
            .and_then(|a| a.fields.get("field_name"))
            .and_then(|f| f.as_str())
            .map(|v| Self(v.to_string()))
            .ok_or_else(|| (StatusCode::BAD_REQUEST, "`field_name` authorizer field is missing"))
    }
}

#[async_trait]
impl<S> FromRequest<S> for AuthorizerFields
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        req.request_context_ref()
            .and_then(|r| r.authorizer())
            .map(|a| Self(a.fields.clone()))
            .ok_or_else(|| (StatusCode::BAD_REQUEST, "authorizer is missing"))
    }
}

async fn extract_field(AuthorizerField(field): AuthorizerField) -> Json<Value> {
    Json(json!({ "field extracted": field }))
}

async fn extract_all_fields(AuthorizerFields(fields): AuthorizerFields) -> Json<Value> {
    Json(json!({ "authorizer fields": fields }))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // If you use API Gateway stages, the Rust Runtime will include the stage name
    // as part of the path that your application receives.
    // Setting the following environment variable, you can remove the stage from the path.
    // This variable only applies to API Gateway stages,
    // you can remove it if you don't use them.
    // i.e with: `GET /test-stage/todo/id/123` without: `GET /todo/id/123`
    set_var("AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH", "true");

    // required to enable CloudWatch error logging by the runtime
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let app = Router::new()
        .route("/extract-field", get(extract_field))
        .route("/extract-all-fields", get(extract_all_fields));

    run(app).await
}
