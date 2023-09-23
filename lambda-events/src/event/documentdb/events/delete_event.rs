use serde::{Deserialize, Serialize};

use super::commom_types::{DocumentId, DocumentKeyId, Timestamp, InsertNs};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeDeleteEvent<> {
    #[serde(rename = "_id")]
    id: DocumentId,
    cluster_time: Timestamp,
    #[serde(default)]
    #[serde(rename = "collectionUUID")]
    collection_uuid: Option<String>,
    document_key: DocumentKeyId,
    #[serde(default)]
    lsid: Option<String>,
    ns: InsertNs,
    operation_type: String,
    #[serde(default)]
    operation_decription: Option<String>,
    #[serde(default)]
    txt_number: Option<String>,
    #[serde(default)]
    wall_time: Option<String>
}