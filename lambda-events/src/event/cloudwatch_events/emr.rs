use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoScalingPolicyStateChange {
    pub resource_id: String,
    pub cluster_id: String,
    pub state: String,
    pub message: String,
    pub scaling_resource_type: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterStateChange {
    pub severity: String,
    pub state_change_reason: String,
    pub name: String,
    pub cluster_id: String,
    pub state: String,
    pub message: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceGroupStateChange {
    pub market: String,
    pub severity: String,
    pub requested_instance_count: String,
    pub instance_type: String,
    pub instance_group_type: String,
    pub instance_group_id: String,
    pub cluster_id: String,
    pub running_instance_count: String,
    pub state: String,
    pub message: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepStatusChange {
    pub severity: String,
    pub action_on_failure: String,
    pub step_id: String,
    pub name: String,
    pub cluster_id: String,
    pub state: String,
    pub message: String,
}
