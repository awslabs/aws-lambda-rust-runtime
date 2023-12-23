use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

/// `CloudWatchAlarm` is the outer structure of an event triggered by a CloudWatch Alarm.
/// For examples of events that come via CloudWatch Alarms,
/// see https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/AlarmThatSendsEmail.html#Lambda-action-payload
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudWatchAlarm<C = Value, R = Value>
where
    C: DeserializeOwned,
    C: Serialize,
    R: DeserializeOwned,
    R: Serialize,
{
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub alarm_arn: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    pub time: DateTime<Utc>,

    #[serde(default, bound = "")]
    pub alarm_data: CloudWatchAlarmData<C, R>,
}

/// `CloudWatchMetricAlarm` is the structure of an event triggered by CloudWatch metric alarms.
#[allow(unused)]
type CloudWatchMetricAlarm<R = Value> = CloudWatchAlarm<CloudWatchMetricAlarmConfiguration, R>;

/// `CloudWatchCompositeAlarm` is the structure of an event triggered by CloudWatch composite alarms.
#[allow(unused)]
type CloudWatchCompositeAlarm<R = Value> = CloudWatchAlarm<CloudWatchCompositeAlarmConfiguration, R>;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudWatchAlarmData<C = Value, R = Value>
where
    C: DeserializeOwned,
    C: Serialize,
    R: DeserializeOwned,
    R: Serialize,
{
    pub alarm_name: String,
    #[serde(default, bound = "")]
    pub state: Option<CloudWatchAlarmState<R>>,
    #[serde(default, bound = "")]
    pub previous_state: Option<CloudWatchAlarmState<R>>,
    #[serde(bound = "")]
    pub configuration: C,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudWatchAlarmState<R = Value>
where
    R: DeserializeOwned,
    R: Serialize,
{
    pub value: String,
    pub reason: String,
    #[serde(default, bound = "")]
    pub reason_data: Option<R>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudWatchMetricAlarmConfiguration {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub metrics: Vec<CloudWatchMetricDefinition>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudWatchMetricDefinition {
    pub id: String,
    #[serde(default)]
    pub return_data: bool,
    pub metric_stat: CloudWatchMetricStatDefinition,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudWatchMetricStatDefinition {
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub stat: Option<String>,
    pub period: u16,
    pub metric: CloudWatchMetricStatMetricDefinition,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudWatchMetricStatMetricDefinition {
    #[serde(default)]
    pub namespace: Option<String>,
    pub name: String,
    pub dimensions: HashMap<String, String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudWatchCompositeAlarmConfiguration {
    pub alarm_rule: String,
    pub actions_suppressor: String,
    pub actions_suppressor_wait_period: u16,
    pub actions_suppressor_extension_period: u16,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CloudWatchAlarmStateValue {
    #[default]
    Ok,
    Alarm,
    InsuficientData,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[cfg(feature = "cloudwatch_alarms")]
    fn example_cloudwatch_alarm_metric() {
        let data = include_bytes!("../../fixtures/example-cloudwatch-alarm-metric.json");
        let parsed: CloudWatchMetricAlarm = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CloudWatchMetricAlarm = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cloudwatch_alarms")]
    fn example_cloudwatch_alarm_composite() {
        let data = include_bytes!("../../fixtures/example-cloudwatch-alarm-composite.json");
        let parsed: CloudWatchCompositeAlarm = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CloudWatchCompositeAlarm = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
