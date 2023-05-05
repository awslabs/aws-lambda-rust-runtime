use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use serde_json::Value;

pub mod cloudtrail;
pub mod codedeploy;
pub mod codepipeline;
pub mod ec2;
pub mod emr;
pub mod gamelift;
pub mod glue;
pub mod health;
pub mod kms;
pub mod macie;
pub mod opsworks;
pub mod signin;
pub mod sms;
pub mod ssm;
pub mod tag;
pub mod trustedadvisor;

/// `CloudWatchEvent` is the outer structure of an event sent via CloudWatch Events.
/// For examples of events that come via CloudWatch Events, see https://docs.aws.amazon.com/AmazonCloudWatch/latest/events/EventTypes.html
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudWatchEvent<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    #[serde(rename = "detail-type")]
    pub detail_type: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    #[serde(rename = "account")]
    pub account_id: Option<String>,
    pub time: DateTime<Utc>,
    #[serde(default)]
    pub region: Option<String>,
    pub resources: Vec<String>,
    #[serde(bound = "")]
    pub detail: Option<T1>,
}
