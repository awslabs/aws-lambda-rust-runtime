use serde::{Deserialize, Serialize};

use super::commom_types::{DocumentId, Timestamp, InsertNs};


// TODO: Campos pendentes, carece insumo de teste

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeDropDatabaseEvent<> {
    #[serde(rename = "_id")]
    id: DocumentId,
    cluster_time: Timestamp,
    #[serde(default)]
    lsid: Option<String>,
    ns: InsertNs,
    operation_type: String,
    #[serde(default)]
    #[serde(rename = "txnNumber")]
    txn_number: Option<String>,
    #[serde(default)]
    #[serde(rename = "wallTime")]
    wall_time: Option<String>
}
