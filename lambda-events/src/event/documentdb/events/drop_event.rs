use serde::{Deserialize, Serialize};

use super::commom_types::{DocumentId, DocumentKeyId, Timestamp, InsertNs};


// TODO: Campos pendentes, carece insumo de teste

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeDropEvent<> {
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
    txt_number: Option<String>,
    #[serde(default)]
    wall_time: Option<String>
}
