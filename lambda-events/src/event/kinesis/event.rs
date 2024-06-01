use crate::encodings::{Base64Data, SecondTimestamp};
use crate::time_window::{TimeWindowEventResponseProperties, TimeWindowProperties};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KinesisEvent {
    #[serde(rename = "Records")]
    pub records: Vec<KinesisEventRecord>,
}

/// `KinesisTimeWindowEvent` represents an Amazon Dynamodb event when using time windows
/// ref. https://docs.aws.amazon.com/lambda/latest/dg/with-kinesis.html#services-kinesis-windows
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KinesisTimeWindowEvent {
    #[serde(rename = "KinesisEvent")]
    #[serde(flatten)]
    pub kinesis_event: KinesisEvent,
    #[serde(rename = "TimeWindowProperties")]
    #[serde(flatten)]
    pub time_window_properties: TimeWindowProperties,
}

/// `KinesisTimeWindowEventResponse` is the outer structure to report batch item failures for KinesisTimeWindowEvent.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KinesisTimeWindowEventResponse {
    #[serde(rename = "TimeWindowEventResponseProperties")]
    #[serde(flatten)]
    pub time_window_event_response_properties: TimeWindowEventResponseProperties,
    // pub batch_item_failures: Vec<KinesisBatchItemFailure>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KinesisEventRecord {
    /// nolint: stylecheck
    #[serde(default)]
    pub aws_region: Option<String>,
    #[serde(default)]
    #[serde(rename = "eventID")]
    pub event_id: Option<String>,
    #[serde(default)]
    pub event_name: Option<String>,
    #[serde(default)]
    pub event_source: Option<String>,
    /// nolint: stylecheck
    #[serde(default)]
    #[serde(rename = "eventSourceARN")]
    pub event_source_arn: Option<String>,
    #[serde(default)]
    pub event_version: Option<String>,
    /// nolint: stylecheck
    #[serde(default)]
    pub invoke_identity_arn: Option<String>,
    pub kinesis: KinesisRecord,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KinesisRecord {
    pub approximate_arrival_timestamp: SecondTimestamp,
    pub data: Base64Data,
    #[serde(default)]
    pub encryption_type: KinesisEncryptionType,
    #[serde(default)]
    pub partition_key: String,
    #[serde(default)]
    pub sequence_number: String,
    #[serde(default)]
    pub kinesis_schema_version: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KinesisEncryptionType {
    #[default]
    None,
    Kms,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[cfg(feature = "kinesis")]
    fn example_kinesis_event() {
        let data = include_bytes!("../../fixtures/example-kinesis-event.json");
        let parsed: KinesisEvent = serde_json::from_slice(data).unwrap();
        assert_eq!(KinesisEncryptionType::None, parsed.records[0].kinesis.encryption_type);

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: KinesisEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "kinesis")]
    fn example_kinesis_event_encrypted() {
        let data = include_bytes!("../../fixtures/example-kinesis-event-encrypted.json");
        let parsed: KinesisEvent = serde_json::from_slice(data).unwrap();
        assert_eq!(KinesisEncryptionType::Kms, parsed.records[0].kinesis.encryption_type);

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: KinesisEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
