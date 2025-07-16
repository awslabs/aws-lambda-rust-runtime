use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;
#[cfg(feature = "catch-all-fields")]
use std::collections::HashMap;

/// `CodePipelineJobEvent` contains data from an event sent from AWS CodePipeline
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodePipelineJobEvent {
    #[serde(rename = "CodePipeline.job")]
    pub code_pipeline_job: CodePipelineJob,
    /// Catchall to catch any additional fields that were present but not expected by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// `CodePipelineJob` represents a job from an AWS CodePipeline event
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodePipelineJob {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    pub data: CodePipelineData,
    /// Catchall to catch any additional fields that were present but not expected by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// `CodePipelineData` represents a job from an AWS CodePipeline event
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodePipelineData {
    pub action_configuration: CodePipelineActionConfiguration,
    pub input_artifacts: Vec<CodePipelineInputArtifact>,
    #[serde(rename = "outputArtifacts")]
    pub out_put_artifacts: Vec<CodePipelineOutputArtifact>,
    pub artifact_credentials: CodePipelineArtifactCredentials,
    #[serde(default)]
    pub continuation_token: Option<String>,
    /// Catchall to catch any additional fields that were present but not expected by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// `CodePipelineActionConfiguration` represents an Action Configuration
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodePipelineActionConfiguration {
    pub configuration: CodePipelineConfiguration,
    /// Catchall to catch any additional fields that were present but not expected by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// `CodePipelineConfiguration` represents a configuration for an Action Configuration
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodePipelineConfiguration {
    #[serde(default)]
    #[serde(rename = "FunctionName")]
    pub function_name: Option<String>,
    #[serde(default)]
    #[serde(rename = "UserParameters")]
    pub user_parameters: Option<String>,
    /// Catchall to catch any additional fields that were present but not expected by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// `CodePipelineInputArtifact` represents an input artifact
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodePipelineInputArtifact {
    pub location: CodePipelineInputLocation,
    pub revision: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    /// Catchall to catch any additional fields that were present but not expected by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// `CodePipelineInputLocation` represents a input location
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodePipelineInputLocation {
    pub s3_location: CodePipelineS3Location,
    #[serde(default)]
    #[serde(rename = "type")]
    pub location_type: Option<String>,
    /// Catchall to catch any additional fields that were present but not expected by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// `CodePipelineS3Location` represents an s3 input location
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodePipelineS3Location {
    #[serde(default)]
    pub bucket_name: Option<String>,
    #[serde(default)]
    pub object_key: Option<String>,
    /// Catchall to catch any additional fields that were present but not expected by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// `CodePipelineOutputArtifact` represents an output artifact
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodePipelineOutputArtifact {
    pub location: CodePipelineInputLocation,
    pub revision: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    /// Catchall to catch any additional fields that were present but not expected by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// `CodePipelineOutputLocation` represents a output location
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodePipelineOutputLocation {
    pub s3_location: CodePipelineS3Location,
    #[serde(default)]
    #[serde(rename = "type")]
    pub location_type: Option<String>,
    /// Catchall to catch any additional fields that were present but not expected by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// `CodePipelineArtifactCredentials` represents CodePipeline artifact credentials
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodePipelineArtifactCredentials {
    #[serde(default)]
    pub secret_access_key: Option<String>,
    #[serde(default)]
    pub session_token: Option<String>,
    #[serde(default)]
    pub access_key_id: Option<String>,
    /// Catchall to catch any additional fields that were present but not expected by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[cfg(feature = "codepipeline_job")]
    fn example_codepipeline_job_event() {
        let data = include_bytes!("../../fixtures/example-codepipeline_job-event.json");
        let parsed: CodePipelineJobEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CodePipelineJobEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
