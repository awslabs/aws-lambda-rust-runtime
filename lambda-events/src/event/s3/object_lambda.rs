use http::HeaderMap;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::custom_serde::{deserialize_headers, serialize_headers};

/// `S3ObjectLambdaEvent` contains data coming from S3 object lambdas
/// See: https://docs.aws.amazon.com/AmazonS3/latest/userguide/olap-writing-lambda.html
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3ObjectLambdaEvent<P = Value>
where
    P: DeserializeOwned,
    P: Serialize,
{
    pub x_amz_request_id: String,
    pub get_object_context: Option<GetObjectContext>,
    pub head_object_context: Option<HeadObjectContext>,
    pub list_objects_context: Option<ListObjectsContext>,
    pub list_objects_v2_context: Option<ListObjectsV2Context>,
    #[serde(default, bound = "")]
    pub configuration: Configuration<P>,
    pub user_request: UserRequest,
    pub user_identity: UserIdentity,
    pub protocol_version: String,
}

/// `GetObjectContext` contains the input and output details
/// for connections to Amazon S3 and S3 Object Lambda
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetObjectContext {
    pub input_s3_url: String,
    pub output_route: String,
    pub output_token: String,
}

/// `HeadObjectContext`
/// for connections to Amazon S3 and S3 Object Lambda
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeadObjectContext {
    pub input_s3_url: String,
}

/// `ListObjectsContext`
/// for connections to Amazon S3 and S3 Object Lambda
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListObjectsContext {
    pub input_s3_url: String,
}

/// `ListObjectsV2Context`
/// for connections to Amazon S3 and S3 Object Lambda
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListObjectsV2Context {
    pub input_s3_url: String,
}

/// `Configuration` contains information about the Object Lambda access point
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration<P = Value>
where
    P: DeserializeOwned,
    P: Serialize,
{
    pub access_point_arn: String,
    pub supporting_access_point_arn: String,
    #[serde(default, bound = "")]
    pub payload: P,
}

/// `UserRequest` contains information about the original call to S3 Object Lambda
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRequest {
    pub url: String,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
}

/// `UserIdentity` contains details about the identity that made the call to S3 Object Lambda
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]

pub struct UserIdentity {
    pub r#type: String,
    pub principal_id: String,
    pub arn: String,
    pub account_id: String,
    pub access_key_id: String,
    pub session_context: Option<SessionContext>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionContext {
    pub attributes: HashMap<String, String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionIssuer {
    pub r#type: String,
    pub principal_id: String,
    pub arn: String,
    pub account_id: String,
    pub user_name: String,
}

#[cfg(test)]
mod test {
    use super::*;

    use serde_json;

    #[test]
    #[cfg(feature = "s3")]
    fn example_object_lambda_event_get_object_assumed_role() {
        let data = include_bytes!("../../fixtures/example-s3-object-lambda-event-get-object-assumed-role.json");
        let parsed: S3ObjectLambdaEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: S3ObjectLambdaEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "s3")]
    fn example_object_lambda_event_get_object_iam() {
        let data = include_bytes!("../../fixtures/example-s3-object-lambda-event-get-object-iam.json");
        let parsed: S3ObjectLambdaEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: S3ObjectLambdaEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "s3")]
    fn example_object_lambda_event_head_object_iam() {
        let data = include_bytes!("../../fixtures/example-s3-object-lambda-event-head-object-iam.json");
        let parsed: S3ObjectLambdaEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: S3ObjectLambdaEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "s3")]
    fn example_object_lambda_event_list_objects_iam() {
        let data = include_bytes!("../../fixtures/example-s3-object-lambda-event-list-objects-iam.json");
        let parsed: S3ObjectLambdaEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: S3ObjectLambdaEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "s3")]
    fn example_object_lambda_event_list_objects_v2_iam() {
        let data = include_bytes!("../../fixtures/example-s3-object-lambda-event-list-objects-v2-iam.json");
        let parsed: S3ObjectLambdaEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: S3ObjectLambdaEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
