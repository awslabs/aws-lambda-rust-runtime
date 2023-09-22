use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::commom_types::{DocumentId, DocumentKeyId, Timestamp};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InsertNs {
    db: String,
    coll: String,
}

// TODO: Campos pendentes, carece insumo de teste

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeInsertEvent<T: Serialize> {
    #[serde(rename = "_id")]
    id: DocumentId,
    cluster_time: Timestamp,
    #[serde(default)]
    #[serde(rename = "collectionUUID")]
    collection_uuid: Option<String>,
    document_key: DocumentKeyId,
    full_document: T,
    ns: InsertNs,
    operation_type: String,
}
