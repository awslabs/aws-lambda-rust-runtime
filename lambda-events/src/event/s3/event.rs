use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::custom_serde::deserialize_lambda_map;

/// `S3Event` which wrap an array of `S3Event`Record
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3Event {
    #[serde(rename = "Records")]
    pub records: Vec<S3EventRecord>,
}

/// `S3EventRecord` which wrap record data
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3EventRecord {
    #[serde(default)]
    pub event_version: Option<String>,
    #[serde(default)]
    pub event_source: Option<String>,
    #[serde(default)]
    pub aws_region: Option<String>,
    pub event_time: DateTime<Utc>,
    #[serde(default)]
    pub event_name: Option<String>,
    #[serde(rename = "userIdentity")]
    pub principal_id: S3UserIdentity,
    pub request_parameters: S3RequestParameters,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub response_elements: HashMap<String, String>,
    pub s3: S3Entity,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3UserIdentity {
    #[serde(default)]
    pub principal_id: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3RequestParameters {
    #[serde(default)]
    #[serde(rename = "sourceIPAddress")]
    pub source_ip_address: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3Entity {
    #[serde(default)]
    #[serde(rename = "s3SchemaVersion")]
    pub schema_version: Option<String>,
    #[serde(default)]
    pub configuration_id: Option<String>,
    pub bucket: S3Bucket,
    pub object: S3Object,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3Bucket {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub owner_identity: Option<S3UserIdentity>,
    #[serde(default)]
    pub arn: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3Object {
    #[serde(default)]
    pub key: Option<String>,
    pub size: Option<i64>,
    #[serde(default)]
    pub url_decoded_key: Option<String>,
    #[serde(default)]
    pub version_id: Option<String>,
    #[serde(default)]
    pub e_tag: Option<String>,
    #[serde(default)]
    pub sequencer: Option<String>,
}

#[cfg(test)]
mod test {
    use super::*;

    use serde_json;

    #[test]
    #[cfg(feature = "s3")]
    fn example_s3_event() {
        let data = include_bytes!("../../fixtures/example-s3-event.json");
        let parsed: S3Event = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: S3Event = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "s3")]
    fn example_s3_event_with_decoded() {
        let data = include_bytes!("../../fixtures/example-s3-event-with-decoded.json");
        let parsed: S3Event = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: S3Event = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
