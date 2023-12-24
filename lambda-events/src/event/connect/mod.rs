use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::custom_serde::deserialize_lambda_map;

/// `ConnectEvent` contains the data structure for a Connect event.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectEvent {
    #[serde(rename = "Details")]
    pub details: ConnectDetails,
    /// The name of the event.
    #[serde(default)]
    #[serde(rename = "Name")]
    pub name: Option<String>,
}

/// `ConnectDetails` holds the details of a Connect event
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectDetails {
    #[serde(rename = "ContactData")]
    pub contact_data: ConnectContactData,
    /// The parameters that have been set in the Connect instance at the time of the Lambda invocation.
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    #[serde(rename = "Parameters")]
    pub parameters: HashMap<String, String>,
}

/// `ConnectContactData` holds all of the contact information for the user that invoked the Connect event.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectContactData {
    /// The custom attributes from Connect that the Lambda function was invoked with.
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    #[serde(rename = "Attributes")]
    pub attributes: HashMap<String, String>,
    #[serde(default)]
    #[serde(rename = "Channel")]
    pub channel: Option<String>,
    #[serde(default)]
    #[serde(rename = "ContactId")]
    pub contact_id: Option<String>,
    #[serde(rename = "CustomerEndpoint")]
    pub customer_endpoint: ConnectEndpoint,
    #[serde(default)]
    #[serde(rename = "InitialContactId")]
    pub initial_contact_id: Option<String>,
    /// Either: INBOUND/OUTBOUND/TRANSFER/CALLBACK
    #[serde(default)]
    #[serde(rename = "InitiationMethod")]
    pub initiation_method: Option<String>,
    #[serde(default)]
    #[serde(rename = "PreviousContactId")]
    pub previous_contact_id: Option<String>,
    #[serde(rename = "Queue", default)]
    pub queue: Option<ConnectQueue>,
    #[serde(rename = "SystemEndpoint")]
    pub system_endpoint: ConnectEndpoint,
    #[serde(default)]
    #[serde(rename = "InstanceARN")]
    pub instance_arn: Option<String>,
}

/// `ConnectEndpoint` represents routing information.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectEndpoint {
    #[serde(default)]
    #[serde(rename = "Address")]
    pub address: Option<String>,
    #[serde(default)]
    #[serde(rename = "Type")]
    pub type_: Option<String>,
}

/// `ConnectQueue` represents a queue object.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectQueue {
    #[serde(default)]
    #[serde(rename = "Name")]
    pub name: Option<String>,
    #[serde(default)]
    #[serde(rename = "ARN")]
    pub arn: Option<String>,
}

pub type ConnectResponse = HashMap<String, String>;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[cfg(feature = "connect")]
    fn example_connect_event() {
        let data = include_bytes!("../../fixtures/example-connect-event.json");
        let parsed: ConnectEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ConnectEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "connect")]
    fn example_connect_event_without_queue() {
        let data = include_bytes!("../../fixtures/example-connect-event-without-queue.json");
        let parsed: ConnectEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ConnectEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
