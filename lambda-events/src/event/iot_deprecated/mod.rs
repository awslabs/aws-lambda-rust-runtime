use crate::iot::*;

/// `IoTCustomAuthorizerRequest` contains data coming in to a custom IoT device gateway authorizer function.
/// Deprecated: Use IoTCoreCustomAuthorizerRequest instead. `IoTCustomAuthorizerRequest` does not correctly model the request schema
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IoTCustomAuthorizerRequest {
    pub http_context: Option<IoTHttpContext>,
    pub mqtt_context: Option<IoTMqttContext>,
    pub tls_context: Option<IoTTlsContext>,
    #[serde(default)]
    #[serde(rename = "token")]
    pub authorization_token: Option<String>,
    #[serde(default)]
    pub token_signature: Option<String>,
}

pub type IoTHttpContext = IoTCoreHttpContext;

pub type IoTMqttContext = IoTCoreMqttContext;

pub type IoTTlsContext = IoTCoreTlsContext;

/// `IoTCustomAuthorizerResponse` represents the expected format of an IoT device gateway authorization response.
/// Deprecated: Use IoTCoreCustomAuthorizerResponse. `IoTCustomAuthorizerResponse` does not correctly model the response schema.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IoTCustomAuthorizerResponse {
    pub is_authenticated: bool,
    #[serde(default)]
    pub principal_id: Option<String>,
    pub disconnect_after_in_seconds: i32,
    pub refresh_after_in_seconds: i32,
    pub policy_documents: Vec<String>,
}
