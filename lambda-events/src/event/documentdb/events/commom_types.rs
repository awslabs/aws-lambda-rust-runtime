use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InsertNs {
    #[serde(default)]
    db: String,
    #[serde(default)]
    coll: String,
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