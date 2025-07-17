use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CMKEvent {
    #[serde(rename = "key-id")]
    pub key_id: String,
    /// Catchall to catch any additional fields that were present but not expected by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}
