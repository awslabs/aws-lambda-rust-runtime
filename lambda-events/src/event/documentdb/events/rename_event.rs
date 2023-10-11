use serde::{Deserialize, Serialize};

use super::commom_types::{AnyDocument, DocumentId, InsertNs, Timestamp};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameTo {
    db: String,
    coll: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]

pub struct ChangeRenameEvent {
    #[serde(rename = "_id")]
    id: DocumentId,
    #[serde(default)]
    cluster_time: Option<Timestamp>,

    #[serde(default)]
    #[serde(rename = "lsid")]
    ls_id: Option<AnyDocument>,
    ns: InsertNs,
    //operation_type: String,
    #[serde(default)]
    txn_number: Option<String>,
    to: RenameTo,
}
