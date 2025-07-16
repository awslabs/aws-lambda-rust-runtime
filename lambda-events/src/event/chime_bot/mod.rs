use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;
#[cfg(feature = "catch-all-fields")]
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChimeBotEvent {
    #[serde(rename = "Sender")]
    pub sender: ChimeBotEventSender,
    #[serde(rename = "Discussion")]
    pub discussion: ChimeBotEventDiscussion,
    #[serde(default)]
    #[serde(rename = "EventType")]
    pub event_type: Option<String>,
    #[serde(rename = "InboundHttpsEndpoint")]
    pub inbound_https_endpoint: Option<ChimeBotEventInboundHttpsEndpoint>,
    #[serde(rename = "EventTimestamp")]
    pub event_timestamp: DateTime<Utc>,
    #[serde(rename = "Message")]
    pub message: Option<String>,
    /// Catchall to catch any additional fields that were present but not expected by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChimeBotEventSender {
    #[serde(default)]
    #[serde(rename = "SenderId")]
    pub sender_id: Option<String>,
    #[serde(default)]
    #[serde(rename = "SenderIdType")]
    pub sender_id_type: Option<String>,
    /// Catchall to catch any additional fields that were present but not expected by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChimeBotEventDiscussion {
    #[serde(default)]
    #[serde(rename = "DiscussionId")]
    pub discussion_id: Option<String>,
    #[serde(default)]
    #[serde(rename = "DiscussionType")]
    pub discussion_type: Option<String>,
    /// Catchall to catch any additional fields that were present but not expected by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChimeBotEventInboundHttpsEndpoint {
    #[serde(default)]
    #[serde(rename = "EndpointType")]
    pub endpoint_type: Option<String>,
    #[serde(default)]
    #[serde(rename = "Url")]
    pub url: Option<String>,
    /// Catchall to catch any additional fields that were present but not expected by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}
