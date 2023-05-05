use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StateChangeNotification {
    pub instance_group_id: String,
    pub region: String,
    pub application: String,
    pub deployment_id: String,
    pub state: String,
    pub deployment_group: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentStateChangeNotification {
    pub instance_id: String,
    pub region: String,
    pub state: String,
    pub application: String,
    pub deployment_id: String,
    pub instance_group_id: String,
    pub deployment_group: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceStateChangeNotification {
    pub pipeline: String,
    pub version: String,
    pub state: String,
    #[serde(rename = "execution-id")]
    pub execution_id: String,
}
