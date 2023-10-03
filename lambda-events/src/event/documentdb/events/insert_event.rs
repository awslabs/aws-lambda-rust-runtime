use serde::{Deserialize, Serialize};

use super::commom_types::{AnyDocument, DocumentId, DocumentKeyId, InsertNs, Timestamp};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeInsertEvent {
    #[serde(rename = "_id")]
    id: DocumentId,
    #[serde(default)]
    cluster_time: Option<Timestamp>,
    document_key: DocumentKeyId,
    #[serde(default)]
    #[serde(rename = "lsid")]
    ls_id: Option<String>,
    ns: InsertNs,
    // operation_type: String,
    #[serde(default)]
    txn_number: Option<AnyDocument>,
}
