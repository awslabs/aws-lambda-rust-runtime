use http::HeaderMap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use aws_lambda_json_impl::Value;
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
    #[serde(default)]
    pub session_issuer: Option<SessionIssuer>,
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

    #[test]
    #[cfg(feature = "s3")]
    fn example_object_lambda_event_get_object_assumed_role() {
        let mut data = include_bytes!("../../fixtures/example-s3-object-lambda-event-get-object-assumed-role.json").to_vec();
        let parsed: S3ObjectLambdaEvent = aws_lambda_json_impl::from_slice(data.as_mut_slice()).unwrap();
        let mut output = aws_lambda_json_impl::to_string(&parsed).unwrap().into_bytes();
        let reparsed: S3ObjectLambdaEvent = aws_lambda_json_impl::from_slice(output.as_mut_slice()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "s3")]
    fn example_object_lambda_event_get_object_iam() {
        let mut data = include_bytes!("../../fixtures/example-s3-object-lambda-event-get-object-iam.json").to_vec();
        let parsed: S3ObjectLambdaEvent = aws_lambda_json_impl::from_slice(data.as_mut_slice()).unwrap();
        let mut output = aws_lambda_json_impl::to_string(&parsed).unwrap().into_bytes();
        let reparsed: S3ObjectLambdaEvent = aws_lambda_json_impl::from_slice(output.as_mut_slice()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "s3")]
    fn example_object_lambda_event_head_object_iam() {
        let mut data = include_bytes!("../../fixtures/example-s3-object-lambda-event-head-object-iam.json").to_vec();
        let parsed: S3ObjectLambdaEvent = aws_lambda_json_impl::from_slice(data.as_mut_slice()).unwrap();
        let mut output = aws_lambda_json_impl::to_string(&parsed).unwrap().into_bytes();
        let reparsed: S3ObjectLambdaEvent = aws_lambda_json_impl::from_slice(output.as_mut_slice()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "s3")]
    fn example_object_lambda_event_list_objects_iam() {
        let mut data = include_bytes!("../../fixtures/example-s3-object-lambda-event-list-objects-iam.json").to_vec();
        let parsed: S3ObjectLambdaEvent = aws_lambda_json_impl::from_slice(data.as_mut_slice()).unwrap();
        let mut output = aws_lambda_json_impl::to_string(&parsed).unwrap().into_bytes();
        let reparsed: S3ObjectLambdaEvent = aws_lambda_json_impl::from_slice(output.as_mut_slice()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "s3")]
    fn example_object_lambda_event_list_objects_v2_iam() {
        let mut data = include_bytes!("../../fixtures/example-s3-object-lambda-event-list-objects-v2-iam.json").to_vec();
        let parsed: S3ObjectLambdaEvent = aws_lambda_json_impl::from_slice(data.as_mut_slice()).unwrap();
        let mut output = aws_lambda_json_impl::to_string(&parsed).unwrap().into_bytes();
        let reparsed: S3ObjectLambdaEvent = aws_lambda_json_impl::from_slice(output.as_mut_slice()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
