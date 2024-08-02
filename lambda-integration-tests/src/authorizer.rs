use std::env;

use aws_lambda_events::{
    apigw::{ApiGatewayCustomAuthorizerPolicy, ApiGatewayCustomAuthorizerResponse},
    event::iam::IamPolicyStatement,
};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::Deserialize;
use serde_json::json;
use tracing_subscriber::EnvFilter;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct APIGatewayCustomAuthorizerRequest {
    authorization_token: String,
    method_arn: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .with_current_span(false)
        .with_span_list(false)
        .without_time()
        .init();
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(
    event: LambdaEvent<APIGatewayCustomAuthorizerRequest>,
) -> Result<ApiGatewayCustomAuthorizerResponse, Error> {
    let expected_token = env::var("SECRET_TOKEN").expect("could not read the secret token");
    if event.payload.authorization_token == expected_token {
        return Ok(allow(&event.payload.method_arn));
    }
    panic!("token is not valid");
}

fn allow(method_arn: &str) -> ApiGatewayCustomAuthorizerResponse {
    let stmt = IamPolicyStatement {
        action: vec!["execute-api:Invoke".to_string()],
        resource: vec![method_arn.to_owned()],
        effect: aws_lambda_events::iam::IamPolicyEffect::Allow,
        condition: None,
    };
    let policy = ApiGatewayCustomAuthorizerPolicy {
        version: Some("2012-10-17".to_string()),
        statement: vec![stmt],
    };
    ApiGatewayCustomAuthorizerResponse {
        principal_id: Some("user".to_owned()),
        policy_document: policy,
        context: json!({ "hello": "world" }),
        usage_identifier_key: None,
    }
}
