use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use aws_lambda_json_impl::Value;

/// Parse EventBridge events.
/// Deserialize the event detail into a structure that's `DeserializeOwned`.
///
/// See https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-events-structure.html for structure details.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(bound(deserialize = "T1: DeserializeOwned"))]
#[serde(rename_all = "kebab-case")]
pub struct EventBridgeEvent<T1 = Value>
where
    T1: Serialize,
    T1: DeserializeOwned,
{
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    pub detail_type: String,
    pub source: String,
    #[serde(default)]
    pub account: Option<String>,
    #[serde(default)]
    pub time: Option<DateTime<Utc>>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub resources: Option<Vec<String>>,
    #[serde(bound = "")]
    pub detail: T1,
}

#[cfg(test)]
#[cfg(feature = "eventbridge")]
mod test {
    use super::*;

    #[test]
    fn example_eventbridge_obj_event() {
        #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
        #[serde(rename_all = "kebab-case")]
        struct Ec2StateChange {
            instance_id: String,
            state: String,
        }

        // Example from https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/monitoring-instance-state-changes.html
        let mut data = include_bytes!("../../fixtures/example-eventbridge-event-obj.json").to_vec();
        let parsed: EventBridgeEvent<Ec2StateChange> = aws_lambda_json_impl::from_slice(data.as_mut_slice()).unwrap();

        assert_eq!("i-abcd1111", parsed.detail.instance_id);
        assert_eq!("pending", parsed.detail.state);

        let mut output = aws_lambda_json_impl::to_string(&parsed).unwrap().into_bytes();
        let reparsed: EventBridgeEvent<Ec2StateChange> = aws_lambda_json_impl::from_slice(output.as_mut_slice()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn example_eventbridge_schedule_event() {
        let mut data = include_bytes!("../../fixtures/example-eventbridge-schedule.json").to_vec();
        let parsed: EventBridgeEvent = aws_lambda_json_impl::from_slice(data.as_mut_slice()).unwrap();

        assert_eq!("aws.events", parsed.source);
        assert_eq!("Scheduled Event", parsed.detail_type);

        let mut output = aws_lambda_json_impl::to_string(&parsed).unwrap().into_bytes();
        let reparsed: EventBridgeEvent = aws_lambda_json_impl::from_slice(output.as_mut_slice()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
