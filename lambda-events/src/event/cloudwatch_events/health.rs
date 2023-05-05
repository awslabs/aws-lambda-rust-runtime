use std::collections::HashMap;

use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub event_arn: String,
    pub service: String,
    pub event_type_code: String,
    pub event_type_category: String,
    pub start_time: String,
    pub end_time: String,
    pub event_description: Vec<EventDescription>,
    pub affected_entities: Option<Vec<Entity>>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventDescription {
    pub language: String,
    pub latest_description: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entity {
    pub entity_value: String,
    pub tags: HashMap<String, String>,
}
