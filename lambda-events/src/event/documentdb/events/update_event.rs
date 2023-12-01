use serde::{Deserialize, Serialize};

use super::commom_types::{AnyDocument, DatabaseCollection, DocumentId, DocumentKeyId, Timestamp};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTruncate {
    field: String,
    new_size: usize,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDescription {
    removed_fields: Vec<String>,
    truncated_arrays: Vec<UpdateTruncate>,
    updated_fields: AnyDocument,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeUpdateEvent {
    #[serde(rename = "_id")]
    id: DocumentId,
    #[serde(default)]
    cluster_time: Option<Timestamp>,
    document_key: DocumentKeyId,
    #[serde(default)]
    #[serde(rename = "lsid")]
    ls_id: Option<AnyDocument>,
    ns: DatabaseCollection,
    // operation_type: String,
    update_description: UpdateDescription,
    #[serde(default)]
    txn_number: Option<AnyDocument>,
}
