use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub type CodePipelineStageState = String;

pub type CodePipelineState = String;

pub type CodePipelineActionState = String;

/// CodePipelineEvent is documented at:
/// https://docs.aws.amazon.com/AmazonCloudWatch/latest/events/EventTypes.html#codepipeline_event_type
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodePipelineCloudWatchEvent {
    /// Version is the version of the event's schema.
    #[serde(default)]
    pub version: Option<String>,
    /// ID is the GUID of this event.
    #[serde(default)]
    pub id: Option<String>,
    /// DetailType informs the schema of the Detail field. For deployment state-change
    /// events, the value should be equal to CodePipelineDeploymentEventDetailType.
    /// For instance state-change events, the value should be equal to
    /// CodePipelineInstanceEventDetailType.
    #[serde(default)]
    #[serde(rename = "detail-type")]
    pub detail_type: Option<String>,
    /// Source should be equal to CodePipelineEventSource.
    #[serde(default)]
    pub source: Option<String>,
    /// AccountID is the id of the AWS account from which the event originated.
    #[serde(default)]
    #[serde(rename = "account")]
    pub account_id: Option<String>,
    /// Time is the event's timestamp.
    pub time: DateTime<Utc>,
    /// Region is the AWS region from which the event originated.
    #[serde(default)]
    pub region: Option<String>,
    /// Resources is a list of ARNs of CodePipeline applications and deployment
    /// groups that this event pertains to.
    pub resources: Vec<String>,
    /// Detail contains information specific to a deployment event.
    pub detail: CodePipelineEventDetail,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodePipelineEventDetail {
    #[serde(default)]
    pub pipeline: Option<String>,
    /// From live testing this is always int64 not string as documented
    pub version: i64,
    #[serde(default)]
    #[serde(rename = "execution-id")]
    pub execution_id: Option<String>,
    #[serde(default)]
    pub stage: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    pub state: CodePipelineState,
    #[serde(default)]
    pub region: Option<String>,
    pub type_: Option<CodePipelineEventDetailType>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodePipelineEventDetailType {
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    /// From published EventBridge schema registry this is always int64 not string as documented
    pub version: i64,
}

#[cfg(test)]
mod test {
    // To save on boiler plate, JSON data is parsed from a mut byte slice rather than an &str. The slice isn't actually mutated
    // when using serde-json, but it IS when using simd-json - so we also take care not to reuse the slice
    // once it has been deserialized.

    use super::*;

    #[test]
    #[cfg(feature = "codepipeline_cloudwatch")]
    fn example_codepipeline_action_execution_stage_change_event() {
        let mut data =
            include_bytes!("../../fixtures/example-codepipeline-action-execution-stage-change-event.json").to_vec();
        let parsed: CodePipelineCloudWatchEvent = aws_lambda_json_impl::from_slice(data.as_mut_slice()).unwrap();
        let mut output = aws_lambda_json_impl::to_string(&parsed).unwrap().into_bytes();
        let reparsed: CodePipelineCloudWatchEvent = aws_lambda_json_impl::from_slice(output.as_mut_slice()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "codepipeline_cloudwatch")]
    fn example_codepipeline_execution_stage_change_event() {
        let mut data = include_bytes!("../../fixtures/example-codepipeline-execution-stage-change-event.json").to_vec();
        let parsed: CodePipelineCloudWatchEvent = aws_lambda_json_impl::from_slice(data.as_mut_slice()).unwrap();
        let mut output = aws_lambda_json_impl::to_string(&parsed).unwrap().into_bytes();
        let reparsed: CodePipelineCloudWatchEvent = aws_lambda_json_impl::from_slice(output.as_mut_slice()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "codepipeline_cloudwatch")]
    fn example_codepipeline_execution_state_change_event() {
        let mut data = include_bytes!("../../fixtures/example-codepipeline-execution-state-change-event.json").to_vec();
        let parsed: CodePipelineCloudWatchEvent = aws_lambda_json_impl::from_slice(data.as_mut_slice()).unwrap();
        let mut output = aws_lambda_json_impl::to_string(&parsed).unwrap().into_bytes();
        let reparsed: CodePipelineCloudWatchEvent = aws_lambda_json_impl::from_slice(output.as_mut_slice()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
