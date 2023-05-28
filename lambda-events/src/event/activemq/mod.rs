use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::custom_serde::deserialize_lambda_map;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveMqEvent {
    #[serde(default)]
    pub event_source: Option<String>,
    #[serde(default)]
    pub event_source_arn: Option<String>,
    pub messages: Vec<ActiveMqMessage>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveMqMessage {
    #[serde(default)]
    #[serde(rename = "messageID")]
    pub message_id: Option<String>,
    #[serde(default)]
    pub message_type: Option<String>,
    pub timestamp: i64,
    pub delivery_mode: i64,
    #[serde(default)]
    #[serde(rename = "correlationID")]
    pub correlation_id: Option<String>,
    #[serde(default)]
    pub reply_to: Option<String>,
    pub destination: ActiveMqDestination,
    pub redelivered: bool,
    #[serde(default)]
    pub type_: Option<String>,
    pub expiration: i64,
    pub priority: i64,
    #[serde(default)]
    pub data: Option<String>,
    pub broker_in_time: i64,
    pub broker_out_time: i64,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveMqDestination {
    #[serde(default)]
    pub physical_name: Option<String>,
}

#[cfg(test)]
mod test {
    use super::*;

    use serde_json;

    #[test]
    #[cfg(feature = "activemq")]
    fn example_activemq_event() {
        let data = include_bytes!("../../fixtures/example-activemq-event.json");
        let parsed: ActiveMqEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ActiveMqEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
