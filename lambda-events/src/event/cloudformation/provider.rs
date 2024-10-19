//! These events are to be used with the CDK custom resource provider framework.
//!
//! Note that they are similar (but not the same) as the events in the `super` module.
//!
//! See https://docs.aws.amazon.com/cdk/api/v2/docs/aws-cdk-lib.custom_resources-readme.html for details.

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use aws_lambda_json_impl::Value;

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
    #[serde(flatten, bound = "")]
    pub common: CommonRequestParams<P2>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateRequest<P1 = Value, P2 = Value>
where
    P1: DeserializeOwned + Serialize,
    P2: DeserializeOwned + Serialize,
{
    pub physical_resource_id: String,

    #[serde(bound = "")]
    pub old_resource_properties: P1,

    #[serde(flatten, bound = "")]
    pub common: CommonRequestParams<P2>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteRequest<P2 = Value>
where
    P2: DeserializeOwned + Serialize,
{
    pub physical_resource_id: String,

    #[serde(flatten, bound = "")]
    pub common: CommonRequestParams<P2>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CommonRequestParams<P2 = Value>
where
    P2: DeserializeOwned + Serialize,
{
    pub logical_resource_id: String,
    #[serde(bound = "")]
    pub resource_properties: P2,
    pub resource_type: String,
    pub request_id: String,
    pub stack_id: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct CloudFormationCustomResourceResponse<D = Value>
where
    D: DeserializeOwned + Serialize,
{
    pub physical_resource_id: Option<String>,
    #[serde(bound = "")]
    pub data: D,
    pub no_echo: bool,
}

#[cfg(test)]
mod test {
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
    fn example_create_request() {
        let mut data = include_bytes!("../../fixtures/example-cloudformation-custom-resource-provider-create-request.json").to_vec();
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
    fn example_update_request() {
        let mut data = include_bytes!("../../fixtures/example-cloudformation-custom-resource-provider-update-request.json").to_vec();
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
    fn example_delete_request() {
        let mut data = include_bytes!("../../fixtures/example-cloudformation-custom-resource-provider-delete-request.json").to_vec();
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
    fn example_response() {
        let mut data = include_bytes!("../../fixtures/example-cloudformation-custom-resource-provider-response.json").to_vec();
        let parsed: CloudFormationCustomResourceResponse = aws_lambda_json_impl::from_slice(data.as_mut_slice()).unwrap();
        let mut output = aws_lambda_json_impl::to_string(&parsed).unwrap().into_bytes();
        let reparsed: CloudFormationCustomResourceResponse = aws_lambda_json_impl::from_slice(output.as_mut_slice()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
