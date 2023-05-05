use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobStateChange {
    pub state: String,
    #[serde(rename = "replication-run-id")]
    pub replication_run_id: String,
    #[serde(rename = "replication-job-id")]
    pub replication_job_id: String,
    #[serde(rename = "ami-id")]
    pub ami_id: Option<String>,
    pub version: String,
}
