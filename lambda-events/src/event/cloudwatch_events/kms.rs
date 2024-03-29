use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CMKEvent {
    #[serde(rename = "key-id")]
    pub key_id: String,
}
