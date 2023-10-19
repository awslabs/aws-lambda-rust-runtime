use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type AnyDocument = HashMap<String, Value>;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseCollection {
    db: String,
    #[serde(default)]
    coll: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct DocumentId {
    #[serde(rename = "_data")]
    pub data: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct DocumentKeyIdOid {
    #[serde(rename = "$oid")]
    pub oid: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct DocumentKeyId {
    #[serde(rename = "_id")]
    pub id: DocumentKeyIdOid,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct InnerTimestamp {
    t: usize,
    i: usize,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Timestamp {
    #[serde(rename = "$timestamp")]
    pub timestamp: InnerTimestamp,
}
