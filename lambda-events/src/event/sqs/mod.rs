use crate::custom_serde::deserialize_lambda_map;
use crate::encodings::Base64Data;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The Event sent to Lambda from SQS. Contains 1 or more individual SQS Messages
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SqsEvent {
    #[serde(rename = "Records")]
    pub records: Vec<SqsMessage>,
}

/// An individual SQS Message, its metadata, and Message Attributes
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SqsMessage {
    /// nolint: stylecheck
    #[serde(default)]
    pub message_id: Option<String>,
    #[serde(default)]
    pub receipt_handle: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub md5_of_body: Option<String>,
    #[serde(default)]
    pub md5_of_message_attributes: Option<String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub attributes: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub message_attributes: HashMap<String, SqsMessageAttribute>,
    #[serde(default)]
    #[serde(rename = "eventSourceARN")]
    pub event_source_arn: Option<String>,
    #[serde(default)]
    pub event_source: Option<String>,
    #[serde(default)]
    pub aws_region: Option<String>,
}

/// Alternative to `SqsEvent` to be used alongside `SqsMessageObj<T>` when you need to deserialize a nested object into a struct of type `T` within the SQS Message rather than just using the raw SQS Message string
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(bound(deserialize = "T: DeserializeOwned"))]
pub struct SqsEventObj<T: Serialize> {
    #[serde(rename = "Records")]
    #[serde(bound(deserialize = "T: DeserializeOwned"))]
    pub records: Vec<SqsMessageObj<T>>,
}

/// Alternative to `SqsMessage` to be used alongside `SqsEventObj<T>` when you need to deserialize a nested object into a struct of type `T` within the SQS Message rather than just using the raw SQS Message string
#[serde_with::serde_as]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(bound(deserialize = "T: DeserializeOwned"))]
#[serde(rename_all = "camelCase")]
pub struct SqsMessageObj<T: Serialize> {
    /// nolint: stylecheck
    #[serde(default)]
    pub message_id: Option<String>,
    #[serde(default)]
    pub receipt_handle: Option<String>,

    /// Deserialized into a `T` from nested JSON inside the SQS body string. `T` must implement the `Deserialize` or `DeserializeOwned` trait.
    #[serde_as(as = "serde_with::json::JsonString")]
    #[serde(bound(deserialize = "T: DeserializeOwned"))]
    pub body: T,
    #[serde(default)]
    pub md5_of_body: Option<String>,
    #[serde(default)]
    pub md5_of_message_attributes: Option<String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub attributes: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub message_attributes: HashMap<String, SqsMessageAttribute>,
    #[serde(default)]
    #[serde(rename = "eventSourceARN")]
    pub event_source_arn: Option<String>,
    #[serde(default)]
    pub event_source: Option<String>,
    #[serde(default)]
    pub aws_region: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SqsMessageAttribute {
    pub string_value: Option<String>,
    pub binary_value: Option<Base64Data>,
    #[serde(default)]
    pub string_list_values: Vec<String>,
    #[serde(default)]
    pub binary_list_values: Vec<Base64Data>,
    #[serde(default)]
    pub data_type: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SqsBatchResponse {
    pub batch_item_failures: Vec<BatchItemFailure>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchItemFailure {
    pub item_identifier: String,
}

#[cfg(test)]
mod test {
    use super::*;

    use serde_json;

    #[test]
    #[cfg(feature = "sqs")]
    fn example_sqs_event() {
        let data = include_bytes!("../../fixtures/example-sqs-event.json");
        let parsed: SqsEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: SqsEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "sqs")]
    fn example_sqs_obj_event() {
        #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
        struct CustStruct {
            a: String,
            b: u32,
        }

        let data = include_bytes!("../../fixtures/example-sqs-event-obj.json");
        let parsed: SqsEventObj<CustStruct> = serde_json::from_slice(data).unwrap();

        assert_eq!(parsed.records[0].body.a, "Test");
        assert_eq!(parsed.records[0].body.b, 123);

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: SqsEventObj<CustStruct> = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "sqs")]
    fn example_sqs_batch_response() {
        // Example sqs batch response fetched 2022-05-13, from:
        // https://docs.aws.amazon.com/lambda/latest/dg/with-sqs.html#services-sqs-batchfailurereporting
        let data = include_bytes!("../../fixtures/example-sqs-batch-response.json");
        let parsed: SqsBatchResponse = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: SqsBatchResponse = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
