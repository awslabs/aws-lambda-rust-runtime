use std::collections::HashMap;

use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EC2AutomationStepStatusChange {
    #[serde(rename = "ExecutionId")]
    pub execution_id: String,
    #[serde(rename = "Definition")]
    pub definition: String,
    #[serde(rename = "DefinitionVersion")]
    pub definition_version: f64,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "EndTime")]
    pub end_time: String,
    #[serde(rename = "StartTime")]
    pub start_time: String,
    #[serde(rename = "Time")]
    pub time: f64,
    #[serde(rename = "StepName")]
    pub step_name: String,
    #[serde(rename = "Action")]
    pub action: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EC2AutomationExecutionStatusChange {
    #[serde(rename = "ExecutionId")]
    pub execution_id: String,
    #[serde(rename = "Definition")]
    pub definition: String,
    #[serde(rename = "DefinitionVersion")]
    pub definition_version: f64,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "StartTime")]
    pub start_time: String,
    #[serde(rename = "EndTime")]
    pub end_time: String,
    #[serde(rename = "Time")]
    pub time: f64,
    #[serde(rename = "ExecutedBy")]
    pub executed_by: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StateChange {
    pub state: String,
    pub at_time: String,
    pub next_transition_time: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurationComplianceStateChange {
    #[serde(rename = "last-runtime")]
    pub last_runtime: Option<String>,
    #[serde(rename = "compliance-status")]
    pub compliance_status: String,
    #[serde(rename = "resource-type")]
    pub resource_type: String,
    #[serde(rename = "resource-id")]
    pub resource_id: String,
    #[serde(rename = "compliance-type")]
    pub compliance_type: String,
    #[serde(rename = "patch-baseline-id")]
    pub patch_baseline_id: Option<String>,
    pub serverity: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaintenanceWindowTargetRegistration {
    #[serde(rename = "window-target-id")]
    pub window_target_id: String,
    #[serde(rename = "window-id")]
    pub window_id: String,
    pub status: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaintenanceWindowExecutionStateChange {
    #[serde(rename = "start-time")]
    pub start_time: String,
    #[serde(rename = "end-time")]
    pub end_time: String,
    #[serde(rename = "window-id")]
    pub window_id: String,
    #[serde(rename = "window-execution-id")]
    pub window_execution_id: String,
    pub status: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaintenanceWindowTaskExecutionStateChange {
    #[serde(rename = "start-time")]
    pub start_time: String,
    #[serde(rename = "task-execution-id")]
    pub task_execution_id: String,
    #[serde(rename = "end-time")]
    pub end_time: String,
    #[serde(rename = "window-id")]
    pub window_id: String,
    #[serde(rename = "window-execution-id")]
    pub window_execution_id: String,
    pub status: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaintenanceWindowTaskTargetInvocationStateChange {
    #[serde(rename = "start-time")]
    pub start_time: String,
    #[serde(rename = "end-time")]
    pub end_time: String,
    #[serde(rename = "window-id")]
    pub window_id: String,
    #[serde(rename = "window-execution-id")]
    pub window_execution_id: String,
    #[serde(rename = "task-execution-id")]
    pub task_execution_id: String,
    #[serde(rename = "window-target-id")]
    pub window_target_id: String,
    pub status: String,
    #[serde(rename = "owner-information")]
    pub owner_information: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaintenanceWindowStateChange {
    #[serde(rename = "window-id")]
    pub window_id: String,
    pub status: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParameterStoreStateChange {
    pub operation: String,
    pub name: String,
    pub r#type: String,
    pub description: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EC2CommandStatusChange {
    #[serde(rename = "command-id")]
    pub command_id: String,
    #[serde(rename = "document-name")]
    pub document_name: String,
    #[serde(rename = "expire-after")]
    pub expire_after: String,
    pub parameters: HashMap<String, String>,
    #[serde(rename = "requested-date-time")]
    pub requested_date_time: String,
    pub status: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EC2CommandInvocationStatusChange {
    #[serde(rename = "command-id")]
    pub command_id: String,
    #[serde(rename = "document-name")]
    pub document_name: String,
    #[serde(rename = "instance-id")]
    pub instance_id: String,
    #[serde(rename = "requested-date-time")]
    pub requested_date_time: String,
    pub status: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EC2StateManagerAssociationStateChange {
    #[serde(rename = "association-id")]
    pub association_id: String,
    #[serde(rename = "document-name")]
    pub document_name: String,
    #[serde(rename = "association-version")]
    pub association_version: String,
    #[serde(rename = "document-version")]
    pub document_version: String,
    pub targets: String,
    #[serde(rename = "creation-date")]
    pub creation_date: String,
    #[serde(rename = "last-successful-execution-date")]
    pub last_successful_execution_date: String,
    #[serde(rename = "last-execution-date")]
    pub last_execution_date: String,
    #[serde(rename = "last-updated-date")]
    pub last_updated_date: String,
    pub status: String,
    #[serde(rename = "association-status-aggregated-count")]
    pub association_status_aggregated_count: String,
    #[serde(rename = "schedule-expression")]
    pub schedule_expression: String,
    #[serde(rename = "association-cwe-version")]
    pub association_cwe_version: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EC2StateManagerInstanceAssociationStateChange {
    #[serde(rename = "association-id")]
    pub association_id: String,
    #[serde(rename = "instance-id")]
    pub instance_id: String,
    #[serde(rename = "document-name")]
    pub document_name: String,
    #[serde(rename = "document-version")]
    pub document_version: String,
    pub targets: String,
    #[serde(rename = "creation-date")]
    pub creation_date: String,
    #[serde(rename = "last-successful-execution-date")]
    pub last_successful_execution_date: String,
    #[serde(rename = "last-execution-date")]
    pub last_execution_date: String,
    pub status: String,
    #[serde(rename = "detailed-status")]
    pub detailed_status: String,
    #[serde(rename = "error-code")]
    pub error_code: String,
    #[serde(rename = "execution-summary")]
    pub execution_summary: String,
    #[serde(rename = "output-url")]
    pub output_url: String,
    #[serde(rename = "instance-association-cwe-version")]
    pub instance_association_cwe_version: String,
}
