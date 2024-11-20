use aws_lambda_json_impl::Value;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    // To save on boiler plate, JSON data is parsed from a mut byte slice rather than an &str. The slice isn't actually mutated
    // when using serde-json, but it IS when using simd-json - so we also take care not to reuse the slice
    // once it has been deserialized.

    use super::AWSAPICall;

    #[test]
    #[cfg(feature = "cloudwatch_events")]
    fn example_cloudwatch_cloudtrail_unknown_assumed_role() {
        let mut data = include_bytes!("../../fixtures/example-cloudwatch-cloudtrail-assumed-role.json").to_vec();
        let parsed: AWSAPICall = aws_lambda_json_impl::from_slice(data.as_mut_slice()).unwrap();
        let mut output = aws_lambda_json_impl::to_string(&parsed).unwrap().into_bytes();
        let reparsed: AWSAPICall = aws_lambda_json_impl::from_slice(output.as_mut_slice()).unwrap();
        assert_eq!(parsed, reparsed);
    }
    #[test]
    #[cfg(feature = "cloudwatch_events")]
    fn example_cloudwatch_cloudtrail_unknown_federate() {
        let mut data = include_bytes!("../../fixtures/example-cloudwatch-cloudtrail-unknown-federate.json").to_vec();
        let parsed: AWSAPICall = aws_lambda_json_impl::from_slice(data.as_mut_slice()).unwrap();
        let mut output = aws_lambda_json_impl::to_string(&parsed).unwrap().into_bytes();
        let reparsed: AWSAPICall = aws_lambda_json_impl::from_slice(output.as_mut_slice()).unwrap();
        assert_eq!(parsed, reparsed);
    }
    #[test]
    #[cfg(feature = "cloudwatch_events")]
    fn example_cloudwatch_cloudtrail_assumed_role() {
        let mut data = include_bytes!("../../fixtures/example-cloudwatch-cloudtrail-unknown-user-auth.json").to_vec();
        let parsed: AWSAPICall = aws_lambda_json_impl::from_slice(data.as_mut_slice()).unwrap();
        let mut output = aws_lambda_json_impl::to_string(&parsed).unwrap().into_bytes();
        let reparsed: AWSAPICall = aws_lambda_json_impl::from_slice(output.as_mut_slice()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
