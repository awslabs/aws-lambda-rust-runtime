use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignIn {
    pub event_version: String,
    pub user_identity: UserIdentity,
    pub event_time: String,
    pub event_source: String,
    pub event_name: String,
    pub aws_region: String,
    #[serde(rename = "sourceIPAddress")]
    pub source_ipaddress: String,
    pub user_agent: String,
    pub request_parameters: Value,
    pub response_elements: ResponseElements,
    pub additional_event_data: AdditionalEventData,
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
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseElements {
    #[serde(rename = "ConsoleLogin")]
    pub console_login: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdditionalEventData {
    #[serde(rename = "LoginTo")]
    pub login_to: String,
    #[serde(rename = "MobileVersion")]
    pub mobile_version: String,
    #[serde(rename = "MFAUsed")]
    pub mfaused: String,
}
