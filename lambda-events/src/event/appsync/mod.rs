use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::custom_serde::deserialize_lambda_map;

/// Deprecated: `AppSyncResolverTemplate` does not represent resolver events sent by AppSync. Instead directly model your input schema, or use map[string]string, json.RawMessage, interface{}, etc..
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncResolverTemplate<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    #[serde(default)]
    pub version: Option<String>,
    pub operation: AppSyncOperation,
    #[serde(bound = "")]
    pub payload: Option<T1>,
}

/// `AppSyncIamIdentity` contains information about the caller authed via IAM.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncIamIdentity {
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub cognito_identity_auth_provider: Option<String>,
    #[serde(default)]
    pub cognito_identity_auth_type: Option<String>,
    #[serde(default)]
    pub cognito_identity_pool_id: Option<String>,
    #[serde(default)]
    pub cognito_identity_id: Option<String>,
    pub source_ip: Vec<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub user_arn: Option<String>,
}

/// `AppSyncCognitoIdentity` contains information about the caller authed via Cognito.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncCognitoIdentity<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    #[serde(default)]
    pub sub: Option<String>,
    #[serde(default)]
    pub issuer: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    #[serde(bound = "")]
    pub claims: HashMap<String, T1>,
    pub source_ip: Vec<String>,
    #[serde(default)]
    pub default_auth_strategy: Option<String>,
}

pub type AppSyncOperation = String;

/// `AppSyncLambdaAuthorizerRequest` contains an authorization request from AppSync.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncLambdaAuthorizerRequest {
    #[serde(default)]
    pub authorization_token: Option<String>,
    pub request_context: AppSyncLambdaAuthorizerRequestContext,
}

/// `AppSyncLambdaAuthorizerRequestContext` contains the parameters of the AppSync invocation which triggered
/// this authorization request.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncLambdaAuthorizerRequestContext<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    #[serde(default)]
    #[serde(rename = "apiId")]
    pub apiid: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub query_string: Option<String>,
    #[serde(default)]
    pub operation_name: Option<String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    #[serde(bound = "")]
    pub variables: HashMap<String, T1>,
}

/// `AppSyncLambdaAuthorizerResponse` represents the expected format of an authorization response to AppSync.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncLambdaAuthorizerResponse<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    pub is_authorized: bool,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    #[serde(bound = "")]
    pub resolver_context: HashMap<String, T1>,
    pub denied_fields: Option<Vec<String>>,
    pub ttl_override: Option<i64>,
}

#[cfg(test)]
mod test {
    use super::*;

    use serde_json;

    #[test]
    #[cfg(feature = "appsync")]
    fn example_appsync_identity_cognito() {
        let data = include_bytes!("../../fixtures/example-appsync-identity-cognito.json");
        let parsed: AppSyncCognitoIdentity = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: AppSyncCognitoIdentity = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "appsync")]
    fn example_appsync_identity_iam() {
        let data = include_bytes!("../../fixtures/example-appsync-identity-iam.json");
        let parsed: AppSyncIamIdentity = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: AppSyncIamIdentity = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "appsync")]
    fn example_appsync_lambda_auth_request() {
        let data = include_bytes!("../../fixtures/example-appsync-lambda-auth-request.json");
        let parsed: AppSyncLambdaAuthorizerRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: AppSyncLambdaAuthorizerRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "appsync")]
    fn example_appsync_lambda_auth_response() {
        let data = include_bytes!("../../fixtures/example-appsync-lambda-auth-response.json");
        let parsed: AppSyncLambdaAuthorizerResponse = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: AppSyncLambdaAuthorizerResponse = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
