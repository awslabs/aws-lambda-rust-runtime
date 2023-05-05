use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceStateChange {
    #[serde(rename = "initiated_by")]
    pub initiated_by: String,
    pub hostname: String,
    #[serde(rename = "stack-id")]
    pub stack_id: String,
    #[serde(rename = "layer-ids")]
    pub layer_ids: Vec<String>,
    #[serde(rename = "instance-id")]
    pub instance_id: String,
    #[serde(rename = "ec2-instance-id")]
    pub ec2_instance_id: String,
    pub status: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandStateChange {
    #[serde(rename = "command-id")]
    pub command_id: String,
    #[serde(rename = "instance-id")]
    pub instance_id: String,
    pub r#type: String,
    pub status: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentStateChange {
    pub duration: i64,
    #[serde(rename = "stack-id")]
    pub stack_id: String,
    #[serde(rename = "instance-ids")]
    pub instance_ids: Vec<String>,
    #[serde(rename = "deployment-id")]
    pub deployment_id: String,
    pub command: String,
    pub status: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    #[serde(rename = "stack-id")]
    pub stack_id: String,
    #[serde(rename = "instance-id")]
    pub instance_id: String,
    pub r#type: String,
    pub message: String,
}
