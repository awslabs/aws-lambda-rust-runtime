use serde_derive::Deserialize;
use serde_derive::Serialize;
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
pub struct UserIdentity {
    pub r#type: String,
    pub principal_id: String,
    pub arn: String,
    pub account_id: String,
    pub session_context: Option<SessionContext>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionContext {
    pub attributes: Attributes,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attributes {
    pub mfa_authenticated: String,
    pub creation_date: String,
}
