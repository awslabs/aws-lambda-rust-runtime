use aws_lambda_json_impl::Value;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;

pub mod provider;

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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CloudFormationCustomResourceResponse {
    pub status: CloudFormationCustomResourceResponseStatus,
    pub reason: Option<String>,
    pub physical_resource_id: String,
    pub stack_id: String,
    pub request_id: String,
    pub logical_resource_id: String,
    pub no_echo: bool,
    pub data: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CloudFormationCustomResourceResponseStatus {
    Success,
    Failed,
}

#[cfg(test)]
mod test {
    // To save on boiler plate, JSON data is parsed from a mut byte slice rather than an &str. The slice isn't actually mutated
    // when using serde-json, but it IS when using simd-json - so we also take care not to reuse the slice
    // once it has been deserialized.

    use std::collections::HashMap;

    use super::{CloudFormationCustomResourceRequest::*, *};

    #[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
    #[serde(rename_all = "PascalCase")]
    struct TestProperties {
        key_1: String,
        key_2: Vec<String>,
        key_3: HashMap<String, String>,
    }

    type TestRequest = CloudFormationCustomResourceRequest<TestProperties, TestProperties>;

    #[test]
    fn example_cloudformation_custom_resource_create_request() {
        let mut data =
            include_bytes!("../../fixtures/example-cloudformation-custom-resource-create-request.json").to_vec();
        let parsed: TestRequest = aws_lambda_json_impl::from_slice(data.as_mut_slice()).unwrap();

        match parsed {
            Create(_) => (),
            _ => panic!("expected Create request"),
        }

        let mut output = aws_lambda_json_impl::to_string(&parsed).unwrap().into_bytes();
        let reparsed: TestRequest = aws_lambda_json_impl::from_slice(output.as_mut_slice()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn example_cloudformation_custom_resource_update_request() {
        let mut data =
            include_bytes!("../../fixtures/example-cloudformation-custom-resource-update-request.json").to_vec();
        let parsed: TestRequest = aws_lambda_json_impl::from_slice(data.as_mut_slice()).unwrap();

        match parsed {
            Update(_) => (),
            _ => panic!("expected Update request"),
        }

        let mut output = aws_lambda_json_impl::to_string(&parsed).unwrap().into_bytes();
        let reparsed: TestRequest = aws_lambda_json_impl::from_slice(output.as_mut_slice()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn example_cloudformation_custom_resource_delete_request() {
        let mut data =
            include_bytes!("../../fixtures/example-cloudformation-custom-resource-delete-request.json").to_vec();
        let parsed: TestRequest = aws_lambda_json_impl::from_slice(data.as_mut_slice()).unwrap();

        match parsed {
            Delete(_) => (),
            _ => panic!("expected Delete request"),
        }

        let mut output = aws_lambda_json_impl::to_string(&parsed).unwrap().into_bytes();
        let reparsed: TestRequest = aws_lambda_json_impl::from_slice(output.as_mut_slice()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn example_cloudformation_custom_resource_response() {
        let mut data = include_bytes!("../../fixtures/example-cloudformation-custom-resource-response.json").to_vec();
        let parsed: CloudFormationCustomResourceResponse =
            aws_lambda_json_impl::from_slice(data.as_mut_slice()).unwrap();
        let mut output = aws_lambda_json_impl::to_string(&parsed).unwrap().into_bytes();
        let reparsed: CloudFormationCustomResourceResponse =
            aws_lambda_json_impl::from_slice(output.as_mut_slice()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
