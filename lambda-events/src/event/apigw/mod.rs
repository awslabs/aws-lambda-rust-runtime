use crate::custom_serde::{
    deserialize_headers, deserialize_lambda_map, deserialize_nullish_boolean, http_method, serialize_headers,
    serialize_multi_value_headers,
};
use crate::encodings::Body;
use http::{HeaderMap, Method};
use query_map::QueryMap;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// `ApiGatewayProxyRequest` contains data coming from the API Gateway proxy
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayProxyRequest<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    /// The resource path defined in API Gateway
    #[serde(default)]
    pub resource: Option<String>,
    /// The url path for the caller
    #[serde(default)]
    pub path: Option<String>,
    #[serde(with = "http_method")]
    pub http_method: Method,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_multi_value_headers")]
    pub multi_value_headers: HeaderMap,
    #[serde(default, deserialize_with = "query_map::serde::standard::deserialize_empty")]
    pub query_string_parameters: QueryMap,
    #[serde(default, deserialize_with = "query_map::serde::standard::deserialize_empty")]
    pub multi_value_query_string_parameters: QueryMap,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub path_parameters: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub stage_variables: HashMap<String, String>,
    #[serde(default)]
    #[serde(bound = "")]
    pub request_context: ApiGatewayProxyRequestContext<T1>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
    pub is_base64_encoded: bool,
}

/// `ApiGatewayProxyResponse` configures the response to be returned by API Gateway for the request
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayProxyResponse {
    pub status_code: i64,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_multi_value_headers")]
    pub multi_value_headers: HeaderMap,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<Body>,
    #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
    pub is_base64_encoded: bool,
}

/// `ApiGatewayProxyRequestContext` contains the information to identify the AWS account and resources invoking the
/// Lambda function. It also includes Cognito identity information for the caller.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayProxyRequestContext<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub resource_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_name: Option<String>,
    #[serde(default)]
    pub stage: Option<String>,
    #[serde(default)]
    pub domain_name: Option<String>,
    #[serde(default)]
    pub domain_prefix: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub identity: ApiGatewayRequestIdentity,
    #[serde(default)]
    pub resource_path: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    #[serde(bound = "")]
    pub authorizer: HashMap<String, T1>,
    #[serde(with = "http_method")]
    pub http_method: Method,
    #[serde(default)]
    pub request_time: Option<String>,
    #[serde(default)]
    pub request_time_epoch: i64,
    /// The API Gateway rest API Id
    #[serde(default)]
    #[serde(rename = "apiId")]
    pub apiid: Option<String>,
}

/// `ApiGatewayV2httpRequest` contains data coming from the new HTTP API Gateway
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2httpRequest {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub route_key: Option<String>,
    #[serde(default)]
    pub raw_path: Option<String>,
    #[serde(default)]
    pub raw_query_string: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookies: Option<Vec<String>>,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
    #[serde(
        default,
        deserialize_with = "query_map::serde::aws_api_gateway_v2::deserialize_empty"
    )]
    #[serde(skip_serializing_if = "QueryMap::is_empty")]
    pub query_string_parameters: QueryMap,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub path_parameters: HashMap<String, String>,
    pub request_context: ApiGatewayV2httpRequestContext,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub stage_variables: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default)]
    pub is_base64_encoded: bool,
}

/// `ApiGatewayV2httpRequestContext` contains the information to identify the AWS account and resources invoking the Lambda function.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2httpRequestContext<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    #[serde(default)]
    pub route_key: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub stage: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(bound = "", default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorizer: Option<ApiGatewayV2httpRequestContextAuthorizerDescription<T1>>,
    /// The API Gateway HTTP API Id
    #[serde(default)]
    #[serde(rename = "apiId")]
    pub apiid: Option<String>,
    #[serde(default)]
    pub domain_name: Option<String>,
    #[serde(default)]
    pub domain_prefix: Option<String>,
    #[serde(default)]
    pub time: Option<String>,
    #[serde(default)]
    pub time_epoch: i64,
    pub http: ApiGatewayV2httpRequestContextHttpDescription,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<ApiGatewayV2httpRequestContextAuthentication>,
}

/// `ApiGatewayV2httpRequestContextAuthorizerDescription` contains authorizer information for the request context.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2httpRequestContextAuthorizerDescription<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt: Option<ApiGatewayV2httpRequestContextAuthorizerJwtDescription>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    #[serde(bound = "")]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub lambda: HashMap<String, T1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iam: Option<ApiGatewayV2httpRequestContextAuthorizerIamDescription>,
}

/// `ApiGatewayV2httpRequestContextAuthorizerJwtDescription` contains JWT authorizer information for the request context.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2httpRequestContextAuthorizerJwtDescription {
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub claims: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
}

/// `ApiGatewayV2httpRequestContextAuthorizerIamDescription` contains IAM information for the request context.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2httpRequestContextAuthorizerIamDescription {
    #[serde(default)]
    pub access_key: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub caller_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cognito_identity: Option<ApiGatewayV2httpRequestContextAuthorizerCognitoIdentity>,
    #[serde(default)]
    pub principal_org_id: Option<String>,
    #[serde(default)]
    pub user_arn: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
}

/// `ApiGatewayV2httpRequestContextAuthorizerCognitoIdentity` contains Cognito identity information for the request context.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2httpRequestContextAuthorizerCognitoIdentity {
    pub amr: Vec<String>,
    #[serde(default)]
    pub identity_id: Option<String>,
    #[serde(default)]
    pub identity_pool_id: Option<String>,
}

/// `ApiGatewayV2httpRequestContextHttpDescription` contains HTTP information for the request context.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2httpRequestContextHttpDescription {
    #[serde(with = "http_method")]
    pub method: Method,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub source_ip: Option<String>,
    #[serde(default)]
    pub user_agent: Option<String>,
}

/// `ApiGatewayV2httpResponse` configures the response to be returned by API Gateway V2 for the request
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2httpResponse {
    pub status_code: i64,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_multi_value_headers")]
    pub multi_value_headers: HeaderMap,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<Body>,
    #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
    pub is_base64_encoded: bool,
    pub cookies: Vec<String>,
}

/// `ApiGatewayRequestIdentity` contains identity information for the request caller.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayRequestIdentity {
    #[serde(default)]
    pub cognito_identity_pool_id: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub cognito_identity_id: Option<String>,
    #[serde(default)]
    pub caller: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub api_key_id: Option<String>,
    #[serde(default)]
    pub access_key: Option<String>,
    #[serde(default)]
    pub source_ip: Option<String>,
    #[serde(default)]
    pub cognito_authentication_type: Option<String>,
    #[serde(default)]
    pub cognito_authentication_provider: Option<String>,
    /// nolint: stylecheck
    #[serde(default)]
    pub user_arn: Option<String>,
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub user: Option<String>,
}

/// `ApiGatewayWebsocketProxyRequest` contains data coming from the API Gateway proxy
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayWebsocketProxyRequest<T1 = Value, T2 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
    T2: DeserializeOwned,
    T2: Serialize,
{
    /// The resource path defined in API Gateway
    #[serde(default)]
    pub resource: Option<String>,
    /// The url path for the caller
    #[serde(default)]
    pub path: Option<String>,
    #[serde(deserialize_with = "http_method::deserialize_optional")]
    #[serde(serialize_with = "http_method::serialize_optional")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub http_method: Option<Method>,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_multi_value_headers")]
    pub multi_value_headers: HeaderMap,
    #[serde(default, deserialize_with = "query_map::serde::standard::deserialize_empty")]
    pub query_string_parameters: QueryMap,
    #[serde(default, deserialize_with = "query_map::serde::standard::deserialize_empty")]
    pub multi_value_query_string_parameters: QueryMap,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub path_parameters: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub stage_variables: HashMap<String, String>,
    #[serde(bound = "", default)]
    pub request_context: ApiGatewayWebsocketProxyRequestContext<T1, T2>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
    pub is_base64_encoded: bool,
}

/// `ApiGatewayWebsocketProxyRequestContext` contains the information to identify
/// the AWS account and resources invoking the Lambda function. It also includes
/// Cognito identity information for the caller.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayWebsocketProxyRequestContext<T1 = Value, T2 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
    T2: DeserializeOwned,
    T2: Serialize,
{
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub resource_id: Option<String>,
    #[serde(default)]
    pub stage: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub identity: ApiGatewayRequestIdentity,
    #[serde(default)]
    pub resource_path: Option<String>,
    #[serde(bound = "")]
    pub authorizer: Option<T1>,
    #[serde(deserialize_with = "http_method::deserialize_optional")]
    #[serde(serialize_with = "http_method::serialize_optional")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub http_method: Option<Method>,
    /// The API Gateway rest API Id
    #[serde(default)]
    #[serde(rename = "apiId")]
    pub apiid: Option<String>,
    pub connected_at: i64,
    #[serde(default)]
    pub connection_id: Option<String>,
    #[serde(default)]
    pub domain_name: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub event_type: Option<String>,
    #[serde(default)]
    pub extended_request_id: Option<String>,
    #[serde(default)]
    pub integration_latency: Option<String>,
    #[serde(default)]
    pub message_direction: Option<String>,
    #[serde(bound = "")]
    pub message_id: Option<T2>,
    #[serde(default)]
    pub request_time: Option<String>,
    pub request_time_epoch: i64,
    #[serde(default)]
    pub route_key: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

/// `ApiGatewayCustomAuthorizerRequestTypeRequestIdentity` contains identity information for the request caller including certificate information if using mTLS.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayCustomAuthorizerRequestTypeRequestIdentity {
    #[serde(default)]
    pub api_key_id: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub source_ip: Option<String>,
    #[serde(default)]
    pub client_cert: Option<ApiGatewayCustomAuthorizerRequestTypeRequestIdentityClientCert>,
}

/// `ApiGatewayCustomAuthorizerRequestTypeRequestIdentityClientCert` contains certificate information for the request caller if using mTLS.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayCustomAuthorizerRequestTypeRequestIdentityClientCert {
    #[serde(default)]
    pub client_cert_pem: Option<String>,
    #[serde(default)]
    #[serde(rename = "issuerDN")]
    pub issuer_dn: Option<String>,
    #[serde(default)]
    pub serial_number: Option<String>,
    #[serde(default)]
    #[serde(rename = "subjectDN")]
    pub subject_dn: Option<String>,
    pub validity: ApiGatewayCustomAuthorizerRequestTypeRequestIdentityClientCertValidity,
}

/// `ApiGatewayCustomAuthorizerRequestTypeRequestIdentityClientCertValidity` contains certificate validity information for the request caller if using mTLS.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayCustomAuthorizerRequestTypeRequestIdentityClientCertValidity {
    #[serde(default)]
    pub not_after: Option<String>,
    #[serde(default)]
    pub not_before: Option<String>,
}

/// `ApiGatewayV2httpRequestContextAuthentication` contains authentication context information for the request caller including client certificate information if using mTLS.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2httpRequestContextAuthentication {
    #[serde(default)]
    pub client_cert: Option<ApiGatewayV2httpRequestContextAuthenticationClientCert>,
}

/// `ApiGatewayV2httpRequestContextAuthenticationClientCert` contains client certificate information for the request caller if using mTLS.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2httpRequestContextAuthenticationClientCert {
    #[serde(default)]
    pub client_cert_pem: Option<String>,
    #[serde(default)]
    #[serde(rename = "issuerDN")]
    pub issuer_dn: Option<String>,
    #[serde(default)]
    pub serial_number: Option<String>,
    #[serde(default)]
    #[serde(rename = "subjectDN")]
    pub subject_dn: Option<String>,
    pub validity: ApiGatewayV2httpRequestContextAuthenticationClientCertValidity,
}

/// `ApiGatewayV2httpRequestContextAuthenticationClientCertValidity` contains client certificate validity information for the request caller if using mTLS.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2httpRequestContextAuthenticationClientCertValidity {
    #[serde(default)]
    pub not_after: Option<String>,
    #[serde(default)]
    pub not_before: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2CustomAuthorizerV1RequestTypeRequestContext {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub resource_id: Option<String>,
    #[serde(default)]
    pub stage: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
    pub identity: ApiGatewayCustomAuthorizerRequestTypeRequestIdentity,
    #[serde(default)]
    pub resource_path: Option<String>,
    #[serde(with = "http_method")]
    pub http_method: Method,
    #[serde(default)]
    #[serde(rename = "apiId")]
    pub apiid: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2CustomAuthorizerV1Request {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub type_: Option<String>,
    /// nolint: stylecheck
    #[serde(default)]
    pub method_arn: Option<String>,
    #[serde(default)]
    pub identity_source: Option<String>,
    #[serde(default)]
    pub authorization_token: Option<String>,
    #[serde(default)]
    pub resource: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(with = "http_method")]
    pub http_method: Method,
    #[serde(deserialize_with = "http_serde::header_map::deserialize", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub query_string_parameters: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub path_parameters: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub stage_variables: HashMap<String, String>,
    pub request_context: ApiGatewayV2CustomAuthorizerV1RequestTypeRequestContext,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2CustomAuthorizerV2Request {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub type_: Option<String>,
    /// nolint: stylecheck
    #[serde(default)]
    pub route_arn: Option<String>,
    pub identity_source: Vec<String>,
    #[serde(default)]
    pub route_key: Option<String>,
    #[serde(default)]
    pub raw_path: Option<String>,
    #[serde(default)]
    pub raw_query_string: Option<String>,
    #[serde(default)]
    pub cookies: Vec<String>,
    #[serde(deserialize_with = "http_serde::header_map::deserialize", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub query_string_parameters: HashMap<String, String>,
    pub request_context: ApiGatewayV2httpRequestContext,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub path_parameters: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub stage_variables: HashMap<String, String>,
}

/// `ApiGatewayCustomAuthorizerContext` represents the expected format of an API Gateway custom authorizer response.
/// Deprecated. Code should be updated to use the Authorizer map from APIGatewayRequestIdentity. Ex: Authorizer["principalId"]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayCustomAuthorizerContext {
    pub principal_id: Option<String>,
    pub string_key: Option<String>,
    pub num_key: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
    pub bool_key: bool,
}

/// `ApiGatewayCustomAuthorizerRequestTypeRequestContext` represents the expected format of an API Gateway custom authorizer response.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayCustomAuthorizerRequestTypeRequestContext {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub resource_id: Option<String>,
    #[serde(default)]
    pub stage: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub identity: Option<ApiGatewayCustomAuthorizerRequestTypeRequestIdentity>,
    #[serde(default)]
    pub resource_path: Option<String>,
    #[serde(deserialize_with = "http_method::deserialize_optional")]
    #[serde(serialize_with = "http_method::serialize_optional")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub http_method: Option<Method>,
    #[serde(default)]
    #[serde(rename = "apiId")]
    pub apiid: Option<String>,
}

/// `ApiGatewayCustomAuthorizerRequest` contains data coming in to a custom API Gateway authorizer function.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayCustomAuthorizerRequest {
    #[serde(default)]
    pub type_: Option<String>,
    #[serde(default)]
    pub authorization_token: Option<String>,
    /// nolint: stylecheck
    #[serde(default)]
    pub method_arn: Option<String>,
}

/// `ApiGatewayCustomAuthorizerRequestTypeRequest` contains data coming in to a custom API Gateway authorizer function.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayCustomAuthorizerRequestTypeRequest {
    #[serde(default)]
    pub type_: Option<String>,
    /// nolint: stylecheck
    #[serde(default)]
    pub method_arn: Option<String>,
    #[serde(default)]
    pub resource: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(deserialize_with = "http_method::deserialize_optional")]
    #[serde(serialize_with = "http_method::serialize_optional")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub http_method: Option<Method>,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_multi_value_headers")]
    pub multi_value_headers: HeaderMap,
    #[serde(default, deserialize_with = "query_map::serde::standard::deserialize_empty")]
    pub query_string_parameters: QueryMap,
    #[serde(default, deserialize_with = "query_map::serde::standard::deserialize_empty")]
    pub multi_value_query_string_parameters: QueryMap,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub path_parameters: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub stage_variables: HashMap<String, String>,
    pub request_context: ApiGatewayCustomAuthorizerRequestTypeRequestContext,
}

/// `ApiGatewayCustomAuthorizerResponse` represents the expected format of an API Gateway authorization response.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayCustomAuthorizerResponse<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    #[serde(default)]
    pub principal_id: Option<String>,
    pub policy_document: ApiGatewayCustomAuthorizerPolicy,
    #[serde(bound = "", default)]
    pub context: T1,
    pub usage_identifier_key: Option<String>,
}

/// `ApiGatewayV2CustomAuthorizerSimpleResponse` represents the simple format of an API Gateway V2 authorization response.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2CustomAuthorizerSimpleResponse<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    pub is_authorized: bool,
    #[serde(bound = "", default)]
    pub context: T1,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2CustomAuthorizerIamPolicyResponse<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    #[serde(default)]
    pub principal_id: Option<String>,
    pub policy_document: ApiGatewayCustomAuthorizerPolicy,
    #[serde(bound = "", default)]
    pub context: T1,
}

/// `ApiGatewayCustomAuthorizerPolicy` represents an IAM policy
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayCustomAuthorizerPolicy {
    #[serde(default)]
    #[serde(rename = "Version")]
    pub version: Option<String>,
    #[serde(rename = "Statement")]
    pub statement: Vec<IamPolicyStatement>,
}

/// `IamPolicyStatement` represents one statement from IAM policy with action, effect and resource
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IamPolicyStatement {
    #[serde(rename = "Action")]
    pub action: Vec<String>,
    #[serde(default)]
    #[serde(rename = "Effect")]
    pub effect: Option<String>,
    #[serde(rename = "Resource")]
    pub resource: Vec<String>,
}

#[cfg(test)]
mod test {
    use super::*;

    use serde_json;

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_custom_auth_request_type_request() {
        let data = include_bytes!("../../fixtures/example-apigw-custom-auth-request-type-request.json");
        let parsed: ApiGatewayCustomAuthorizerRequestTypeRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayCustomAuthorizerRequestTypeRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_custom_auth_request_type_request_websocket() {
        let data = include_bytes!("../../fixtures/example-apigw-v2-custom-authorizer-websocket-request.json");
        let parsed: ApiGatewayCustomAuthorizerRequestTypeRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayCustomAuthorizerRequestTypeRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_custom_auth_request() {
        let data = include_bytes!("../../fixtures/example-apigw-custom-auth-request.json");
        let parsed: ApiGatewayCustomAuthorizerRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayCustomAuthorizerRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_custom_auth_response() {
        let data = include_bytes!("../../fixtures/example-apigw-custom-auth-response.json");
        let parsed: ApiGatewayCustomAuthorizerResponse = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayCustomAuthorizerResponse = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_request() {
        let data = include_bytes!("../../fixtures/example-apigw-request.json");
        let parsed: ApiGatewayProxyRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayProxyRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_response() {
        let data = include_bytes!("../../fixtures/example-apigw-response.json");
        let parsed: ApiGatewayProxyResponse = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayProxyResponse = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_restapi_openapi_request() {
        let data = include_bytes!("../../fixtures/example-apigw-restapi-openapi-request.json");
        let parsed: ApiGatewayProxyRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayProxyRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_v2_request_iam() {
        let data = include_bytes!("../../fixtures/example-apigw-v2-request-iam.json");
        let parsed: ApiGatewayV2httpRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayV2httpRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_v2_request_jwt_authorizer() {
        let data = include_bytes!("../../fixtures/example-apigw-v2-request-jwt-authorizer.json");
        let parsed: ApiGatewayV2httpRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayV2httpRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_v2_request_lambda_authorizer() {
        let data = include_bytes!("../../fixtures/example-apigw-v2-request-lambda-authorizer.json");
        let parsed: ApiGatewayV2httpRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayV2httpRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_v2_request_no_authorizer() {
        let data = include_bytes!("../../fixtures/example-apigw-v2-request-no-authorizer.json");
        let parsed: ApiGatewayV2httpRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayV2httpRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_websocket_request() {
        let data = include_bytes!("../../fixtures/example-apigw-websocket-request.json");
        let parsed: ApiGatewayWebsocketProxyRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayWebsocketProxyRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_console_test_request() {
        let data = include_bytes!("../../fixtures/example-apigw-console-test-request.json");
        let parsed: ApiGatewayProxyRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayProxyRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_websocket_request_without_method() {
        let data = include_bytes!("../../fixtures/example-apigw-websocket-request-without-method.json");
        let parsed: ApiGatewayWebsocketProxyRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayWebsocketProxyRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_v2_custom_authorizer_v1_request() {
        let data = include_bytes!("../../fixtures/example-apigw-v2-custom-authorizer-v1-request.json");
        let parsed: ApiGatewayV2httpRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayV2httpRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_v2_custom_authorizer_v2_request() {
        let data = include_bytes!("../../fixtures/example-apigw-v2-custom-authorizer-v2-request.json");
        let parsed: ApiGatewayV2CustomAuthorizerV2Request = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayV2CustomAuthorizerV2Request = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_v2_custom_authorizer_v2_request_without_cookies() {
        let data = include_bytes!("../../fixtures/example-apigw-v2-custom-authorizer-v2-request-without-cookies.json");
        let parsed: ApiGatewayV2CustomAuthorizerV2Request = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayV2CustomAuthorizerV2Request = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_console_request() {
        let data = include_bytes!("../../fixtures/example-apigw-console-request.json");
        let parsed: ApiGatewayProxyRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayProxyRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
