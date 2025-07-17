use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;

use super::commom_types::{DocumentId, Timestamp};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeInvalidateEvent {
    #[serde(rename = "_id")]
    id: DocumentId,
    #[serde(default)]
    cluster_time: Option<Timestamp>,
    // operation_type: String,
    /// Catchall to catch any additional fields that were present but not expected by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}
