use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;
#[cfg(feature = "catch-all-fields")]
use std::collections::HashMap;

use super::commom_types::{AnyDocument, DatabaseCollection, DocumentId, DocumentKeyId, Timestamp};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeDeleteEvent {
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
    #[serde(default)]
    txn_number: Option<String>,
    /// Catchall to catch any additional fields that were present but not expected by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}
