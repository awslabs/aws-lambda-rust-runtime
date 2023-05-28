use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagChangeOnResource {
    #[serde(rename = "changed-tag-keys")]
    pub changed_tag_keys: Vec<String>,
    pub service: String,
    #[serde(rename = "resource-type")]
    pub resource_type: String,
    pub version: i64,
    pub tags: HashMap<String, String>,
}
