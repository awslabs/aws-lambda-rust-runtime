use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckItemRefreshNotification {
    #[serde(rename = "check-name")]
    pub check_name: String,
    #[serde(rename = "check-item-detail")]
    pub check_item_detail: HashMap<String, String>,
    pub status: String,
    #[serde(rename = "resource_id")]
    pub resource_id: String,
    pub uuid: String,
}
