use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::custom_serde::deserialize_lambda_map;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LexEvent {
    pub message_version: Option<String>,
    pub invocation_source: Option<String>,
    pub user_id: Option<String>,
    pub input_transcript: Option<String>,
    pub session_attributes: Option<SessionAttributes>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub request_attributes: HashMap<String, String>,
    pub bot: Option<LexBot>,
    pub output_dialog_mode: Option<String>,
    pub current_intent: Option<LexCurrentIntent>,
    pub alternative_intents: Option<Vec<LexAlternativeIntents>>,
    /// Deprecated: the DialogAction field is never populated by Lex events
    pub dialog_action: Option<LexDialogAction>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LexBot {
    pub name: Option<String>,
    pub alias: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LexCurrentIntent {
    pub name: Option<String>,
    pub nlu_intent_confidence_score: Option<f64>,
    pub slots: Option<Slots>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub slot_details: HashMap<String, SlotDetail>,
    pub confirmation_status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LexAlternativeIntents {
    pub name: Option<String>,
    pub nlu_intent_confidence_score: Option<f64>,
    pub slots: Option<Slots>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub slot_details: HashMap<String, SlotDetail>,
    pub confirmation_status: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SlotDetail {
    pub resolutions: Option<Vec<HashMap<String, String>>>,
    pub original_value: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LexDialogAction {
    pub type_: Option<String>,
    pub fulfillment_state: Option<String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub message: HashMap<String, String>,
    pub intent_name: Option<String>,
    pub slots: Option<Slots>,
    pub slot_to_elicit: Option<String>,
    pub response_card: Option<LexResponseCard>,
}

pub type SessionAttributes = HashMap<String, String>;

pub type Slots = HashMap<String, Option<String>>;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LexResponse {
    pub session_attributes: SessionAttributes,
    pub dialog_action: Option<LexDialogAction>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LexResponseCard {
    pub version: Option<i64>,
    pub content_type: Option<String>,
    pub generic_attachments: Option<Vec<Attachment>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub title: Option<String>,
    pub sub_title: Option<String>,
    pub image_url: Option<String>,
    pub attachment_link_url: Option<String>,
    pub buttons: Option<Vec<HashMap<String, String>>>,
}

#[cfg(test)]
mod test {
    use super::*;

    use serde_json;

    #[test]
    #[cfg(feature = "lex")]
    fn example_lex_event() {
        let data = include_bytes!("../../fixtures/example-lex-event.json");
        let parsed: LexEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: LexEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "lex")]
    fn example_lex_response() {
        let data = include_bytes!("../../fixtures/example-lex-response.json");
        let parsed: LexEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: LexEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
