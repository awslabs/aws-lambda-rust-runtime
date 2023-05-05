use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineExecutionStateChange {
    pub pipeline: String,
    pub version: String,
    pub state: String,
    #[serde(rename = "execution-id")]
    pub execution_id: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StageExecutionStateChange {
    pub pipeline: String,
    pub version: String,
    #[serde(rename = "execution-id")]
    pub execution_id: String,
    pub stage: String,
    pub state: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionExecutionStateChange {
    pub pipeline: String,
    pub version: i64,
    #[serde(rename = "execution-id")]
    pub execution_id: String,
    pub stage: String,
    pub action: String,
    pub state: String,
    pub region: String,
    #[serde(rename = "type")]
    pub type_field: ActionExecutionStateChangeType,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionExecutionStateChangeType {
    pub owner: String,
    pub category: String,
    pub provider: String,
    pub version: i64,
}
