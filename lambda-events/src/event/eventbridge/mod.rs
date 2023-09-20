use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct EventBridgeEvent {
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
    #[serde(default)]
    pub detail: Option<String>,
}

#[serde_with::serde_as]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(bound(deserialize = "T: DeserializeOwned"))]
#[serde(rename_all = "kebab-case")]
pub struct EventBridgeEventObj<T: Serialize> {
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
    #[serde_as(as = "serde_with::json::JsonString")]
    #[serde(bound(deserialize = "T: DeserializeOwned"))]
    pub detail: T,
}

#[cfg(test)]
#[cfg(feature = "eventbridge")]
mod test {
    use super::*;

    use serde_json;

    #[test]
    fn example_eventbridge_obj_event() {
        #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
        struct CustomStruct {
            a: String,
            b: String,
        }

        let data = include_bytes!("../../fixtures/example-eventbridge-event-obj.json");
        let parsed: EventBridgeEventObj<CustomStruct> = serde_json::from_slice(data).unwrap();

        assert_eq!(parsed.detail.a, "123");
        assert_eq!(parsed.detail.b, "456");

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: EventBridgeEventObj<CustomStruct> = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn example_eventbridge_event() {
        let data = include_bytes!("../../fixtures/example-eventbridge-event.json");
        let parsed: EventBridgeEvent = serde_json::from_slice(data).unwrap();
        assert_eq!(parsed.detail, Some(String::from("String Message")));

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: EventBridgeEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
