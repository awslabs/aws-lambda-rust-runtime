use std::collections::HashMap;

use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Alert<T> {
    #[serde(rename = "notification-type")]
    pub notification_type: String,
    pub name: String,
    pub tags: Vec<String>,
    pub url: String,
    #[serde(rename = "alert-arn")]
    pub alert_arn: String,
    #[serde(rename = "risk-score")]
    pub risk_score: i64,
    pub trigger: Trigger,
    #[serde(rename = "created-at")]
    pub created_at: String,
    pub actor: String,
    pub summary: T,
}

pub type BucketScanAlert = Alert<BucketScanSummary>;
pub type BucketWritableAlert = Alert<BucketWritableSummary>;
pub type BucketContainsHighRiskObjectAlert = Alert<BucketContainsHighRiskObjectSummary>;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Trigger {
    #[serde(rename = "rule-arn")]
    pub rule_arn: String,
    #[serde(rename = "alert-type")]
    pub alert_type: String,
    #[serde(rename = "created-at")]
    pub created_at: String,
    pub description: String,
    pub risk: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketScanSummary {
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "IP")]
    pub ip: Ip,
    #[serde(rename = "Time Range")]
    pub time_range: Vec<TimeRange>,
    #[serde(rename = "Source ARN")]
    pub source_arn: String,
    #[serde(rename = "Record Count")]
    pub record_count: i64,
    #[serde(rename = "Location")]
    pub location: Location,
    #[serde(rename = "Event Count")]
    pub event_count: i64,
    #[serde(rename = "Events")]
    pub events: HashMap<String, ActionInfo>,
    pub recipient_account_id: HashMap<String, i64>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ip {
    #[serde(rename = "34.199.185.34")]
    pub n34_199_185_34: i64,
    #[serde(rename = "34.205.153.2")]
    pub n34_205_153_2: i64,
    #[serde(rename = "72.21.196.70")]
    pub n72_21_196_70: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeRange {
    pub count: i64,
    pub start: String,
    pub end: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    #[serde(rename = "us-east-1")]
    pub us_east_1: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionInfo {
    pub count: i64,
    #[serde(rename = "ISP")]
    pub isp: HashMap<String, i64>,
    pub error_code: Option<HashMap<String, i64>>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketWritableSummary {
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "Bucket")]
    pub bucket: Bucket,
    #[serde(rename = "Record Count")]
    pub record_count: i64,
    #[serde(rename = "ACL")]
    pub acl: Acl,
    #[serde(rename = "Event Count")]
    pub event_count: i64,
    #[serde(rename = "Timestamps")]
    pub timestamps: HashMap<String, i64>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bucket {
    #[serde(rename = "secret-bucket-name")]
    pub secret_bucket_name: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Acl {
    #[serde(rename = "secret-bucket-name")]
    pub secret_bucket_name: Vec<SecretBucketName>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretBucketName {
    #[serde(rename = "Owner")]
    pub owner: Owner,
    #[serde(rename = "Grants")]
    pub grants: Vec<Grant>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Owner {
    #[serde(rename = "DisplayName")]
    pub display_name: String,
    #[serde(rename = "ID")]
    pub id: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Grant {
    #[serde(rename = "Grantee")]
    pub grantee: Grantee,
    #[serde(rename = "Permission")]
    pub permission: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Grantee {
    pub r#type: String,
    #[serde(rename = "URI")]
    pub uri: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketContainsHighRiskObjectSummary {
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "Object")]
    pub object: HashMap<String, i64>,
    #[serde(rename = "Record Count")]
    pub record_count: i64,
    #[serde(rename = "Themes")]
    pub themes: HashMap<String, i64>,
    #[serde(rename = "Event Count")]
    pub event_count: i64,
    #[serde(rename = "DLP risk")]
    pub dlp_risk: HashMap<String, i64>,
    #[serde(rename = "Owner")]
    pub owner: HashMap<String, i64>,
    #[serde(rename = "Timestamps")]
    pub timestamps: HashMap<String, i64>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertUpdated {
    #[serde(rename = "notification-type")]
    pub notification_type: String,
    pub name: String,
    pub tags: Vec<String>,
    pub url: String,
    #[serde(rename = "alert-arn")]
    pub alert_arn: String,
    #[serde(rename = "risk-score")]
    pub risk_score: i64,
    #[serde(rename = "created-at")]
    pub created_at: String,
    pub actor: String,
    pub trigger: UpdatedTrigger,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatedTrigger {
    #[serde(rename = "alert-type")]
    pub alert_type: String,
    pub features: HashMap<String, FeatureInfo>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureInfo {
    pub name: String,
    pub description: String,
    pub narrative: String,
    pub anomalous: bool,
    pub multiplier: f64,
    #[serde(rename = "excession_times")]
    pub excession_times: Vec<String>,
    pub risk: i64,
}
