use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::custom_serde::deserialize_lambda_map;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RabbitMqEvent {
    #[serde(default)]
    pub event_source: Option<String>,
    #[serde(default)]
    pub event_source_arn: Option<String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    #[serde(rename = "rmqMessagesByQueue")]
    pub messages_by_queue: HashMap<String, Vec<RabbitMqMessage>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RabbitMqMessage {
    pub basic_properties: RabbitMqBasicProperties,
    #[serde(default)]
    pub data: Option<String>,
    pub redelivered: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RabbitMqBasicProperties<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    #[serde(default)]
    pub content_type: Option<String>,
    pub content_encoding: Option<String>,
    /// Application or header exchange table
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    #[serde(bound = "")]
    pub headers: HashMap<String, T1>,
    pub delivery_mode: u8,
    pub priority: u8,
    pub correlation_id: Option<String>,
    pub reply_to: Option<String>,
    #[serde(default)]
    pub expiration: Option<String>,
    pub message_id: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
    pub type_: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    pub app_id: Option<String>,
    pub cluster_id: Option<String>,
    pub body_size: u64,
}

#[cfg(test)]
mod test {
    use super::*;

    use serde_json;

    #[test]
    #[cfg(feature = "rabbitmq")]
    fn example_rabbitmq_event() {
        let data = include_bytes!("../../fixtures/example-rabbitmq-event.json");
        let parsed: RabbitMqEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: RabbitMqEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
