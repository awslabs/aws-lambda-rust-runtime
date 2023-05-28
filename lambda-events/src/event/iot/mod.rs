use crate::custom_serde::serialize_headers;
use crate::encodings::Base64Data;
use crate::iam::IamPolicyDocument;
use http::HeaderMap;
use serde::{Deserialize, Serialize};

/// `IoTCoreCustomAuthorizerRequest` represents the request to an IoT Core custom authorizer.
/// See https://docs.aws.amazon.com/iot/latest/developerguide/config-custom-auth.html
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IoTCoreCustomAuthorizerRequest {
    #[serde(default)]
    pub token: Option<String>,
    pub signature_verified: bool,
    pub protocols: Vec<String>,
    pub protocol_data: Option<IoTCoreProtocolData>,
    pub connection_metadata: Option<IoTCoreConnectionMetadata>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IoTCoreProtocolData {
    pub tls: Option<IoTCoreTlsContext>,
    pub http: Option<IoTCoreHttpContext>,
    pub mqtt: Option<IoTCoreMqttContext>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IoTCoreTlsContext {
    #[serde(default)]
    pub server_name: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IoTCoreHttpContext {
    #[serde(deserialize_with = "http_serde::header_map::deserialize", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
    #[serde(default)]
    pub query_string: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IoTCoreMqttContext {
    #[serde(default)]
    pub client_id: Option<String>,
    pub password: Base64Data,
    #[serde(default)]
    pub username: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IoTCoreConnectionMetadata {
    #[serde(default)]
    pub id: Option<String>,
}

/// `IoTCoreCustomAuthorizerResponse` represents the response from an IoT Core custom authorizer.
/// See https://docs.aws.amazon.com/iot/latest/developerguide/config-custom-auth.html
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IoTCoreCustomAuthorizerResponse {
    pub is_authenticated: bool,
    #[serde(default)]
    pub principal_id: Option<String>,
    pub disconnect_after_in_seconds: u32,
    pub refresh_after_in_seconds: u32,
    pub policy_documents: Vec<Option<IamPolicyDocument>>,
}

#[cfg(test)]
mod test {
    use super::*;

    use serde_json;

    #[test]
    #[cfg(feature = "iot")]
    fn example_iot_custom_auth_request() {
        let data = include_bytes!("../../fixtures/example-iot-custom-auth-request.json");
        let parsed: IoTCoreCustomAuthorizerRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: IoTCoreCustomAuthorizerRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "iot")]
    fn example_iot_custom_auth_response() {
        let data = include_bytes!("../../fixtures/example-iot-custom-auth-response.json");
        let parsed: IoTCoreCustomAuthorizerResponse = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: IoTCoreCustomAuthorizerResponse = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
