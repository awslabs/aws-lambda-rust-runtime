use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::custom_serde::{deserialize_lambda_map, deserialize_nullish_boolean};

/// `CognitoEvent` contains data from an event sent from AWS Cognito Sync
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEvent {
    #[serde(default)]
    pub dataset_name: Option<String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub dataset_records: HashMap<String, CognitoDatasetRecord>,
    #[serde(default)]
    pub event_type: Option<String>,
    #[serde(default)]
    pub identity_id: Option<String>,
    #[serde(default)]
    pub identity_pool_id: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    pub version: i64,
}

/// `CognitoDatasetRecord` represents a record from an AWS Cognito Sync event
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoDatasetRecord {
    #[serde(default)]
    pub new_value: Option<String>,
    #[serde(default)]
    pub old_value: Option<String>,
    #[serde(default)]
    pub op: Option<String>,
}

/// `CognitoEventUserPoolsPreSignup` is sent by AWS Cognito User Pools when a user attempts to register
/// (sign up), allowing a Lambda to perform custom validation to accept or deny the registration request
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPreSignup {
    #[serde(rename = "CognitoEventUserPoolsHeader")]
    #[serde(flatten)]
    pub cognito_event_user_pools_header: CognitoEventUserPoolsHeader,
    pub request: CognitoEventUserPoolsPreSignupRequest,
    pub response: CognitoEventUserPoolsPreSignupResponse,
}

/// `CognitoEventUserPoolsPreAuthentication` is sent by AWS Cognito User Pools when a user submits their information
/// to be authenticated, allowing you to perform custom validations to accept or deny the sign in request.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPreAuthentication {
    #[serde(rename = "CognitoEventUserPoolsHeader")]
    #[serde(flatten)]
    pub cognito_event_user_pools_header: CognitoEventUserPoolsHeader,
    pub request: CognitoEventUserPoolsPreAuthenticationRequest,
    pub response: CognitoEventUserPoolsPreAuthenticationResponse,
}

/// `CognitoEventUserPoolsPostConfirmation` is sent by AWS Cognito User Pools after a user is confirmed,
/// allowing the Lambda to send custom messages or add custom logic.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPostConfirmation {
    #[serde(rename = "CognitoEventUserPoolsHeader")]
    #[serde(flatten)]
    pub cognito_event_user_pools_header: CognitoEventUserPoolsHeader,
    pub request: CognitoEventUserPoolsPostConfirmationRequest,
    pub response: CognitoEventUserPoolsPostConfirmationResponse,
}

/// `CognitoEventUserPoolsPreTokenGen` is sent by AWS Cognito User Pools when a user attempts to retrieve
/// credentials, allowing a Lambda to perform insert, suppress or override claims
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPreTokenGen {
    #[serde(rename = "CognitoEventUserPoolsHeader")]
    #[serde(flatten)]
    pub cognito_event_user_pools_header: CognitoEventUserPoolsHeader,
    pub request: CognitoEventUserPoolsPreTokenGenRequest,
    pub response: CognitoEventUserPoolsPreTokenGenResponse,
}

/// `CognitoEventUserPoolsPostAuthentication` is sent by AWS Cognito User Pools after a user is authenticated,
/// allowing the Lambda to add custom logic.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPostAuthentication {
    #[serde(rename = "CognitoEventUserPoolsHeader")]
    #[serde(flatten)]
    pub cognito_event_user_pools_header: CognitoEventUserPoolsHeader,
    pub request: CognitoEventUserPoolsPostAuthenticationRequest,
    pub response: CognitoEventUserPoolsPostAuthenticationResponse,
}

/// `CognitoEventUserPoolsMigrateUser` is sent by AWS Cognito User Pools when a user does not exist in the
/// user pool at the time of sign-in with a password, or in the forgot-password flow.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsMigrateUser {
    #[serde(rename = "CognitoEventUserPoolsHeader")]
    #[serde(flatten)]
    pub cognito_event_user_pools_header: CognitoEventUserPoolsHeader,
    #[serde(rename = "request")]
    pub cognito_event_user_pools_migrate_user_request: CognitoEventUserPoolsMigrateUserRequest,
    #[serde(rename = "response")]
    pub cognito_event_user_pools_migrate_user_response: CognitoEventUserPoolsMigrateUserResponse,
}

/// `CognitoEventUserPoolsCallerContext` contains information about the caller
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsCallerContext {
    #[serde(default)]
    #[serde(rename = "awsSdkVersion")]
    pub awssdk_version: Option<String>,
    #[serde(default)]
    pub client_id: Option<String>,
}

/// `CognitoEventUserPoolsHeader` contains common data from events sent by AWS Cognito User Pools
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsHeader {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub trigger_source: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub user_pool_id: Option<String>,
    pub caller_context: CognitoEventUserPoolsCallerContext,
    #[serde(default)]
    pub user_name: Option<String>,
}

/// `CognitoEventUserPoolsPreSignupRequest` contains the request portion of a PreSignup event
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPreSignupRequest {
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub user_attributes: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub validation_data: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub client_metadata: HashMap<String, String>,
}

/// `CognitoEventUserPoolsPreSignupResponse` contains the response portion of a PreSignup event
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPreSignupResponse {
    pub auto_confirm_user: bool,
    pub auto_verify_email: bool,
    pub auto_verify_phone: bool,
}

/// `CognitoEventUserPoolsPreAuthenticationRequest` contains the request portion of a PreAuthentication event
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPreAuthenticationRequest {
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub user_attributes: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub validation_data: HashMap<String, String>,
}

/// `CognitoEventUserPoolsPreAuthenticationResponse` contains the response portion of a PreAuthentication event
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct CognitoEventUserPoolsPreAuthenticationResponse {}
/// `CognitoEventUserPoolsPostConfirmationRequest` contains the request portion of a PostConfirmation event
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPostConfirmationRequest {
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub user_attributes: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub client_metadata: HashMap<String, String>,
}

/// `CognitoEventUserPoolsPostConfirmationResponse` contains the response portion of a PostConfirmation event
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct CognitoEventUserPoolsPostConfirmationResponse {}
/// `CognitoEventUserPoolsPreTokenGenRequest` contains request portion of PreTokenGen event
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPreTokenGenRequest {
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub user_attributes: HashMap<String, String>,
    pub group_configuration: GroupConfiguration,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub client_metadata: HashMap<String, String>,
}

/// `CognitoEventUserPoolsPreTokenGenResponse` contains the response portion of  a PreTokenGen event
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPreTokenGenResponse {
    pub claims_override_details: Option<ClaimsOverrideDetails>,
}

/// `CognitoEventUserPoolsPostAuthenticationRequest` contains the request portion of a PostAuthentication event
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPostAuthenticationRequest {
    pub new_device_used: bool,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub user_attributes: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub client_metadata: HashMap<String, String>,
}

/// `CognitoEventUserPoolsPostAuthenticationResponse` contains the response portion of a PostAuthentication event
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct CognitoEventUserPoolsPostAuthenticationResponse {}
/// `CognitoEventUserPoolsMigrateUserRequest` contains the request portion of a MigrateUser event
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsMigrateUserRequest {
    #[serde(default)]
    pub password: Option<String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub validation_data: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub client_metadata: HashMap<String, String>,
}

/// `CognitoEventUserPoolsMigrateUserResponse` contains the response portion of a MigrateUser event
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsMigrateUserResponse {
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub user_attributes: HashMap<String, String>,
    #[serde(default)]
    pub final_user_status: Option<String>,
    #[serde(default)]
    pub message_action: Option<String>,
    pub desired_delivery_mediums: Vec<String>,
    pub force_alias_creation: bool,
}

/// `ClaimsOverrideDetails` allows lambda to add, suppress or override claims in the token
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimsOverrideDetails {
    pub group_override_details: GroupConfiguration,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub claims_to_add_or_override: HashMap<String, String>,
    pub claims_to_suppress: Vec<String>,
}

/// `GroupConfiguration` allows lambda to override groups, roles and set a preferred role
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupConfiguration {
    pub groups_to_override: Vec<String>,
    pub iam_roles_to_override: Vec<String>,
    pub preferred_role: Option<String>,
}

/// `CognitoEventUserPoolsChallengeResult` represents a challenge that is presented to the user in the authentication
/// process that is underway, along with the corresponding result.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsChallengeResult {
    #[serde(default)]
    pub challenge_name: Option<String>,
    pub challenge_result: bool,
    #[serde(default)]
    pub challenge_metadata: Option<String>,
}

/// `CognitoEventUserPoolsDefineAuthChallengeRequest` defines auth challenge request parameters
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsDefineAuthChallengeRequest {
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub user_attributes: HashMap<String, String>,
    pub session: Vec<Option<CognitoEventUserPoolsChallengeResult>>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub client_metadata: HashMap<String, String>,
    #[serde(default)]
    pub user_not_found: bool,
}

/// `CognitoEventUserPoolsDefineAuthChallengeResponse` defines auth challenge response parameters
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsDefineAuthChallengeResponse {
    #[serde(default)]
    pub challenge_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
    pub issue_tokens: bool,
    #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
    pub fail_authentication: bool,
}

/// `CognitoEventUserPoolsDefineAuthChallenge` sent by AWS Cognito User Pools to initiate custom authentication flow
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsDefineAuthChallenge {
    #[serde(rename = "CognitoEventUserPoolsHeader")]
    #[serde(flatten)]
    pub cognito_event_user_pools_header: CognitoEventUserPoolsHeader,
    pub request: CognitoEventUserPoolsDefineAuthChallengeRequest,
    pub response: CognitoEventUserPoolsDefineAuthChallengeResponse,
}

/// `CognitoEventUserPoolsCreateAuthChallengeRequest` defines create auth challenge request parameters
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsCreateAuthChallengeRequest {
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub user_attributes: HashMap<String, String>,
    #[serde(default)]
    pub challenge_name: Option<String>,
    pub session: Vec<Option<CognitoEventUserPoolsChallengeResult>>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub client_metadata: HashMap<String, String>,
}

/// `CognitoEventUserPoolsCreateAuthChallengeResponse` defines create auth challenge response parameters
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsCreateAuthChallengeResponse {
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub public_challenge_parameters: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub private_challenge_parameters: HashMap<String, String>,
    #[serde(default)]
    pub challenge_metadata: Option<String>,
}

/// `CognitoEventUserPoolsCreateAuthChallenge` sent by AWS Cognito User Pools to create a challenge to present to the user
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsCreateAuthChallenge {
    #[serde(rename = "CognitoEventUserPoolsHeader")]
    #[serde(flatten)]
    pub cognito_event_user_pools_header: CognitoEventUserPoolsHeader,
    pub request: CognitoEventUserPoolsCreateAuthChallengeRequest,
    pub response: CognitoEventUserPoolsCreateAuthChallengeResponse,
}

/// `CognitoEventUserPoolsVerifyAuthChallengeRequest` defines verify auth challenge request parameters
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsVerifyAuthChallengeRequest<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub user_attributes: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub private_challenge_parameters: HashMap<String, String>,
    #[serde(bound = "")]
    pub challenge_answer: Option<T1>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub client_metadata: HashMap<String, String>,
}

/// `CognitoEventUserPoolsVerifyAuthChallengeResponse` defines verify auth challenge response parameters
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsVerifyAuthChallengeResponse {
    #[serde(default)]
    pub answer_correct: bool,
}

/// `CognitoEventUserPoolsVerifyAuthChallenge` sent by AWS Cognito User Pools to verify if the response from the end user
/// for a custom Auth Challenge is valid or not
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsVerifyAuthChallenge {
    #[serde(rename = "CognitoEventUserPoolsHeader")]
    #[serde(flatten)]
    pub cognito_event_user_pools_header: CognitoEventUserPoolsHeader,
    pub request: CognitoEventUserPoolsVerifyAuthChallengeRequest,
    pub response: CognitoEventUserPoolsVerifyAuthChallengeResponse,
}

/// `CognitoEventUserPoolsCustomMessage` is sent by AWS Cognito User Pools before a verification or MFA message is sent,
/// allowing a user to customize the message dynamically.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsCustomMessage {
    #[serde(rename = "CognitoEventUserPoolsHeader")]
    #[serde(flatten)]
    pub cognito_event_user_pools_header: CognitoEventUserPoolsHeader,
    pub request: CognitoEventUserPoolsCustomMessageRequest,
    pub response: CognitoEventUserPoolsCustomMessageResponse,
}

/// `CognitoEventUserPoolsCustomMessageRequest` contains the request portion of a CustomMessage event
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsCustomMessageRequest<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    #[serde(bound = "")]
    pub user_attributes: HashMap<String, T1>,
    #[serde(default)]
    pub code_parameter: Option<String>,
    #[serde(default)]
    pub username_parameter: Option<String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub client_metadata: HashMap<String, String>,
}

/// `CognitoEventUserPoolsCustomMessageResponse` contains the response portion of a CustomMessage event
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsCustomMessageResponse {
    #[serde(default)]
    pub sms_message: Option<String>,
    #[serde(default)]
    pub email_message: Option<String>,
    #[serde(default)]
    pub email_subject: Option<String>,
}

#[cfg(test)]
mod test {
    use super::*;

    use serde_json;

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event() {
        let data = include_bytes!("../../fixtures/example-cognito-event.json");
        let parsed: CognitoEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_create_auth_challenge() {
        let data = include_bytes!("../../fixtures/example-cognito-event-userpools-create-auth-challenge.json");
        let parsed: CognitoEventUserPoolsCreateAuthChallenge = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsCreateAuthChallenge = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_custommessage() {
        let data = include_bytes!("../../fixtures/example-cognito-event-userpools-custommessage.json");
        let parsed: CognitoEventUserPoolsCustomMessage = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsCustomMessage = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_define_auth_challenge() {
        let data = include_bytes!("../../fixtures/example-cognito-event-userpools-define-auth-challenge.json");
        let parsed: CognitoEventUserPoolsDefineAuthChallenge = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsDefineAuthChallenge = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_define_auth_challenge_optional_response_fields() {
        let data = include_bytes!(
            "../../fixtures/example-cognito-event-userpools-define-auth-challenge-optional-response-fields.json"
        );
        let parsed: CognitoEventUserPoolsDefineAuthChallenge = serde_json::from_slice(data).unwrap();

        assert!(!parsed.response.fail_authentication);
        assert!(!parsed.response.issue_tokens);

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsDefineAuthChallenge = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_migrateuser() {
        let data = include_bytes!("../../fixtures/example-cognito-event-userpools-migrateuser.json");
        let parsed: CognitoEventUserPoolsMigrateUser = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsMigrateUser = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_postauthentication() {
        let data = include_bytes!("../../fixtures/example-cognito-event-userpools-postauthentication.json");
        let parsed: CognitoEventUserPoolsPostAuthentication = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsPostAuthentication = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_postconfirmation() {
        let data = include_bytes!("../../fixtures/example-cognito-event-userpools-postconfirmation.json");
        let parsed: CognitoEventUserPoolsPostConfirmation = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsPostConfirmation = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_preauthentication() {
        let data = include_bytes!("../../fixtures/example-cognito-event-userpools-preauthentication.json");
        let parsed: CognitoEventUserPoolsPreAuthentication = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsPreAuthentication = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_presignup() {
        let data = include_bytes!("../../fixtures/example-cognito-event-userpools-presignup.json");
        let parsed: CognitoEventUserPoolsPreSignup = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsPreSignup = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_pretokengen_incoming() {
        let data = include_bytes!("../../fixtures/example-cognito-event-userpools-pretokengen-incoming.json");
        let parsed: CognitoEventUserPoolsPreTokenGen = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsPreTokenGen = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_pretokengen() {
        let data = include_bytes!("../../fixtures/example-cognito-event-userpools-pretokengen.json");
        let parsed: CognitoEventUserPoolsPreTokenGen = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsPreTokenGen = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_verify_auth_challenge() {
        let data = include_bytes!("../../fixtures/example-cognito-event-userpools-verify-auth-challenge.json");
        let parsed: CognitoEventUserPoolsVerifyAuthChallenge = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsVerifyAuthChallenge = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_verify_auth_challenge_optional_answer_correct() {
        let data = include_bytes!(
            "../../fixtures/example-cognito-event-userpools-verify-auth-challenge-optional-answer-correct.json"
        );
        let parsed: CognitoEventUserPoolsVerifyAuthChallenge = serde_json::from_slice(data).unwrap();

        assert!(!parsed.response.answer_correct);

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsVerifyAuthChallenge = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
