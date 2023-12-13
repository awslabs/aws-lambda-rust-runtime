use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Parse EventBridge events.
/// Deserialize the event detail into a structure that's `DeserializeOwned`.
///
/// See https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-events-structure.html for structure details.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(bound(deserialize = "T: DeserializeOwned"))]
#[serde(rename_all = "kebab-case")]
pub struct EventBridgeEvent<T: Serialize> {
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
    #[serde(bound(deserialize = "T: DeserializeOwned"))]
    pub detail: T,
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
        let data = include_bytes!("../../fixtures/example-eventbridge-event-obj.json");
        let parsed: EventBridgeEvent<Ec2StateChange> = serde_json::from_slice(data).unwrap();

        assert_eq!("i-abcd1111", parsed.detail.instance_id);
        assert_eq!("pending", parsed.detail.state);

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: EventBridgeEvent<Ec2StateChange> = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
