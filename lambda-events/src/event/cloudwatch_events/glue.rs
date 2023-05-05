use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobRunStateChange {
    pub job_name: String,
    pub severity: String,
    pub state: String,
    pub job_run_id: String,
    pub message: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrawlerStarted {
    pub account_id: String,
    pub crawler_name: String,
    pub start_time: String,
    pub state: String,
    pub message: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrawlerSucceeded {
    pub tables_created: String,
    pub warning_message: String,
    pub partitions_updated: String,
    pub tables_updated: String,
    pub message: String,
    pub partitions_deleted: String,
    pub account_id: String,
    #[serde(rename = "runningTime (sec)")]
    pub running_time_sec: String,
    pub tables_deleted: String,
    pub crawler_name: String,
    pub completion_date: String,
    pub state: String,
    pub partitions_created: String,
    pub cloud_watch_log_link: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrawlerFailed {
    pub crawler_name: String,
    pub error_message: String,
    pub account_id: String,
    pub cloud_watch_log_link: String,
    pub state: String,
    pub message: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobRunStatus {
    pub job_name: String,
    pub severity: String,
    pub notification_condition: NotificationCondition,
    pub state: String,
    pub job_run_id: String,
    pub message: String,
    pub started_on: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationCondition {
    #[serde(rename = "NotifyDelayAfter")]
    pub notify_delay_after: f64,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataCatalogTableStateChange {
    pub database_name: String,
    pub changed_partitions: Vec<String>,
    pub type_of_change: String,
    pub table_name: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataCatalogDatabaseStateChange {
    pub database_name: String,
    pub type_of_change: String,
    pub changed_tables: Vec<String>,
}
