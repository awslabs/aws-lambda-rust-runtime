use serde::{Deserialize, Serialize};

use crate::custom_serde::deserialize_nullish_boolean;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchmakingSearching {
    pub tickets: Vec<Ticket>,
    pub estimated_wait_millis: String,
    pub r#type: String,
    pub game_session_info: GameSessionInfo,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ticket {
    pub ticket_id: String,
    pub start_time: String,
    pub players: Vec<Player>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub player_id: String,
    pub team: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
    pub accepted: bool,
    pub player_session_id: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSessionInfo {
    pub players: Vec<Player>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PotentialMatchCreated {
    pub tickets: Vec<Ticket>,
    pub acceptance_timeout: i64,
    pub rule_evaluation_metrics: Vec<RuleEvaluationMetric>,
    pub acceptance_required: bool,
    pub r#type: String,
    pub game_session_info: GameSessionInfo,
    pub match_id: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleEvaluationMetric {
    pub rule_name: String,
    pub passed_count: i64,
    pub failed_count: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptMatch {
    pub tickets: Vec<Ticket>,
    pub r#type: String,
    pub game_session_info: GameSessionInfo,
    pub match_id: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptMatchCompleted {
    pub tickets: Vec<Ticket>,
    pub acceptance: String,
    pub r#type: String,
    pub game_session_info: GameSessionInfo,
    pub match_id: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchmakingSucceeded {
    pub tickets: Vec<Ticket>,
    pub r#type: String,
    pub game_session_info: GameSessionInfo,
    pub match_id: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchmakingTimedOut {
    pub reason: String,
    pub tickets: Vec<Ticket>,
    pub rule_evaluation_metrics: Vec<RuleEvaluationMetric>,
    pub r#type: String,
    pub message: String,
    pub game_session_info: GameSessionInfo,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchmakingCancelled {
    pub reason: String,
    pub tickets: Vec<Ticket>,
    pub rule_evaluation_metrics: Vec<RuleEvaluationMetric>,
    pub r#type: String,
    pub message: String,
    pub game_session_info: GameSessionInfo,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchmakingFailed {
    pub tickets: Vec<Ticket>,
    pub custom_event_data: String,
    pub r#type: String,
    pub reason: String,
    pub message: String,
    pub game_session_info: GameSessionInfo,
    pub match_id: String,
}
