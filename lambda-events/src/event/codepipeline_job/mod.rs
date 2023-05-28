use serde::{Deserialize, Serialize};

/// `CodePipelineJobEvent` contains data from an event sent from AWS CodePipeline
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodePipelineJobEvent {
    #[serde(rename = "CodePipeline.job")]
    pub code_pipeline_job: CodePipelineJob,
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
}

/// `CodePipelineActionConfiguration` represents an Action Configuration
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodePipelineActionConfiguration {
    pub configuration: CodePipelineConfiguration,
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
}

/// `CodePipelineInputArtifact` represents an input artifact
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodePipelineInputArtifact {
    pub location: CodePipelineInputLocation,
    pub revision: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

/// `CodePipelineInputLocation` represents a input location
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodePipelineInputLocation {
    pub s3_location: CodePipelineS3Location,
    #[serde(default)]
    #[serde(rename = "type")]
    pub location_type: Option<String>,
}

/// `CodePipelineS3Location` represents an s3 input location
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodePipelineS3Location {
    #[serde(default)]
    pub bucket_name: Option<String>,
    #[serde(default)]
    pub object_key: Option<String>,
}

/// `CodePipelineOutputArtifact` represents an output artifact
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodePipelineOutputArtifact {
    pub location: CodePipelineInputLocation,
    pub revision: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

/// `CodePipelineOutputLocation` represents a output location
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodePipelineOutputLocation {
    pub s3_location: CodePipelineS3Location,
    #[serde(default)]
    #[serde(rename = "type")]
    pub location_type: Option<String>,
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
}

#[cfg(test)]
mod test {
    use super::*;

    use serde_json;

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
