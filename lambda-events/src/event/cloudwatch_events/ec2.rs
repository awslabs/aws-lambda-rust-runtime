use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceStateChange {
    #[serde(rename = "instance-id")]
    pub instance_id: String,
    pub state: String,
}
