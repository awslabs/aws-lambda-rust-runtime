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

    #[test]
    #[cfg(feature = "activemq")]
    fn example_activemq_event() {
        let mut data = include_bytes!("../../fixtures/example-activemq-event.json").to_vec();
        let mut data = data.to_vec(); 
        let parsed: ActiveMqEvent = aws_lambda_json_impl::from_slice(data.as_mut_slice()).unwrap();
        let mut output = aws_lambda_json_impl::to_string(&parsed).unwrap().into_bytes();
        let reparsed: ActiveMqEvent = aws_lambda_json_impl::from_slice(output.as_mut_slice()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
