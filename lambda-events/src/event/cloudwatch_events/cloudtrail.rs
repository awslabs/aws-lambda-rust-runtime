use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AWSAPICall<I = Value, O = Value> {
    pub event_version: String,
    pub user_identity: UserIdentity,
    pub event_time: String,
    pub event_source: String,
    pub event_name: String,
    pub aws_region: String,
    #[serde(rename = "sourceIPAddress")]
    pub source_ipaddress: String,
    pub user_agent: String,
    pub request_parameters: I,
    pub response_elements: Option<O>,
    #[serde(default)]
    pub additional_event_data: Option<Value>,
    #[serde(rename = "requestID")]
    pub request_id: String,
    #[serde(rename = "eventID")]
    pub event_id: String,
    pub event_type: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionIssuer {
    pub r#type: String,
    pub user_name: Option<String>,
    pub principal_id: String,
    pub arn: String,
    pub account_id: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebIdFederationData {
    pub federated_provider: Option<String>,
    pub attributes: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attributes {
    pub mfa_authenticated: String,
    pub creation_date: DateTime<Utc>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionContext {
    pub session_issuer: Option<SessionIssuer>,
    pub web_id_federation_data: Option<WebIdFederationData>,
    pub attributes: Attributes,
    pub source_identity: Option<String>,
    pub ec2_role_delivery: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnBehalfOf {
    pub user_id: String,
    pub identity_store_arn: String,
}

// https://docs.aws.amazon.com/awscloudtrail/latest/userguide/cloudtrail-event-reference-user-identity.html
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserIdentity {
    pub r#type: String,
    pub account_id: Option<String>,
    pub arn: Option<String>,
    pub credential_id: Option<String>,
    pub invoked_by: Option<String>,
    pub principal_id: Option<String>,
    pub session_context: Option<SessionContext>,
    pub user_name: Option<String>,
    pub on_behalf_of: Option<OnBehalfOf>,
}
