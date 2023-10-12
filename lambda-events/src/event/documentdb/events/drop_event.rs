use super::commom_types::{AnyDocument, DatabaseCollection, DocumentId, DocumentKeyId, Timestamp};
use serde::{Deserialize, Serialize};

// TODO: Campos pendentes, carece insumo de teste

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeDropEvent {
    #[serde(rename = "_id")]
    id: DocumentId,
    cluster_time: Timestamp,
    #[serde(default)]
    document_key: Option<DocumentKeyId>,
    #[serde(default)]
    #[serde(rename = "lsid")]
    ls_id: Option<AnyDocument>,
    ns: DatabaseCollection,
    // operation_type: String,
    #[serde(default)]
    txn_number: Option<String>,
}
