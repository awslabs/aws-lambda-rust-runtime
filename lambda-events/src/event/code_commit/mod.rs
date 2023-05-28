use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::custom_serde::deserialize_nullish_boolean;

/// `CodeCommitEvent` represents a CodeCommit event
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeCommitEvent {
    #[serde(rename = "Records")]
    pub records: Vec<CodeCommitRecord>,
}

pub type CodeCommitEventTime = DateTime<Utc>;

/// `CodeCommitRecord` represents a CodeCommit record
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeCommitRecord {
    #[serde(default)]
    pub event_id: Option<String>,
    #[serde(default)]
    pub event_version: Option<String>,
    pub event_time: CodeCommitEventTime,
    #[serde(default)]
    pub event_trigger_name: Option<String>,
    pub event_part_number: u64,
    #[serde(rename = "codecommit")]
    pub code_commit: CodeCommitCodeCommit,
    #[serde(default)]
    pub event_name: Option<String>,
    /// nolint: stylecheck
    #[serde(default)]
    pub event_trigger_config_id: Option<String>,
    #[serde(default)]
    #[serde(rename = "eventSourceARN")]
    pub event_source_arn: Option<String>,
    #[serde(default)]
    #[serde(rename = "userIdentityARN")]
    pub user_identity_arn: Option<String>,
    #[serde(default)]
    pub event_source: Option<String>,
    #[serde(default)]
    pub aws_region: Option<String>,
    pub event_total_parts: u64,
    pub custom_data: Option<String>,
}

/// `CodeCommitCodeCommit` represents a CodeCommit object in a record
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeCommitCodeCommit {
    pub references: Vec<CodeCommitReference>,
}

/// `CodeCommitReference` represents a Reference object in a CodeCommit object
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeCommitReference {
    #[serde(default)]
    pub commit: Option<String>,
    #[serde(default)]
    pub ref_: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
    pub created: bool,
}

#[cfg(test)]
mod test {
    use super::*;

    use serde_json;

    #[test]
    #[cfg(feature = "code_commit")]
    fn example_code_commit_event() {
        let data = include_bytes!("../../fixtures/example-code_commit-event.json");
        let parsed: CodeCommitEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CodeCommitEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
