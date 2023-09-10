use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "RequestType")]
pub enum CloudFormationCustomResourceRequest<P1 = Value, P2 = Value>
where
    P1: DeserializeOwned + Serialize,
    P2: DeserializeOwned + Serialize,
{
    #[serde(bound = "")]
    Create(CreateRequest<P2>),
    #[serde(bound = "")]
    Update(UpdateRequest<P1, P2>),
    #[serde(bound = "")]
    Delete(DeleteRequest<P2>),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct CreateRequest<P2 = Value>
where
    P2: DeserializeOwned + Serialize,
{
    #[serde(default)]
    pub service_token: Option<String>,
    pub request_id: String,
    #[serde(rename = "ResponseURL")]
    pub response_url: String,
    pub stack_id: String,
    pub resource_type: String,
    pub logical_resource_id: String,
    #[serde(bound = "")]
    pub resource_properties: P2,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct UpdateRequest<P1 = Value, P2 = Value>
where
    P1: DeserializeOwned + Serialize,
    P2: DeserializeOwned + Serialize,
{
    #[serde(default)]
    pub service_token: Option<String>,
    pub request_id: String,
    #[serde(rename = "ResponseURL")]
    pub response_url: String,
    pub stack_id: String,
    pub resource_type: String,
    pub logical_resource_id: String,
    pub physical_resource_id: String,
    #[serde(bound = "")]
    pub resource_properties: P2,
    #[serde(bound = "")]
    pub old_resource_properties: P1,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
#[serde(deny_unknown_fields)]
pub struct DeleteRequest<P2 = Value>
where
    P2: DeserializeOwned + Serialize,
{
    #[serde(default)]
    pub service_token: Option<String>,
    pub request_id: String,
    #[serde(rename = "ResponseURL")]
    pub response_url: String,
    pub stack_id: String,
    pub resource_type: String,
    pub logical_resource_id: String,
    pub physical_resource_id: String,
    #[serde(bound = "")]
    pub resource_properties: P2,
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::CloudFormationCustomResourceRequest::*;
    use super::*;

    #[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
    #[serde(rename_all = "PascalCase")]
    #[serde(deny_unknown_fields)]
    struct TestProperties {
        key_1: String,
        key_2: Vec<String>,
        key_3: HashMap<String, String>,
    }

    type TestRequest = CloudFormationCustomResourceRequest<TestProperties, TestProperties>;

    #[test]
    fn example_cloudformation_custom_resource_create_request() {
        let data = include_bytes!("../../fixtures/example-cloudformation-custom-resource-create-request.json");
        let parsed: TestRequest = serde_json::from_slice(data).unwrap();

        let Create(_) = parsed else { panic!("expected Create request") };

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: TestRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn example_cloudformation_custom_resource_update_request() {
        let data = include_bytes!("../../fixtures/example-cloudformation-custom-resource-update-request.json");
        let parsed: TestRequest = serde_json::from_slice(data).unwrap();

        let Update(_) = parsed else { panic!("expected Update request") };

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: TestRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn example_cloudformation_custom_resource_delete_request() {
        let data = include_bytes!("../../fixtures/example-cloudformation-custom-resource-delete-request.json");
        let parsed: TestRequest = serde_json::from_slice(data).unwrap();

        let Delete(_) = parsed else { panic!("expected Delete request") };

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: TestRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
