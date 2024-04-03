use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::custom_serde::{deserialize_string_or_slice, iam::deserialize_policy_condition};

/// `IamPolicyDocument` represents an IAM policy document.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct IamPolicyDocument {
    #[serde(default)]
    pub version: Option<String>,
    pub statement: Vec<IamPolicyStatement>,
}

/// `IamPolicyStatement` represents one statement from IAM policy with action, effect and resource
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct IamPolicyStatement {
    #[serde(deserialize_with = "deserialize_string_or_slice")]
    pub action: Vec<String>,
    #[serde(default = "default_statement_effect")]
    pub effect: IamPolicyEffect,
    #[serde(deserialize_with = "deserialize_string_or_slice")]
    pub resource: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_policy_condition")]
    pub condition: Option<IamPolicyCondition>,
}

pub type IamPolicyCondition = HashMap<String, HashMap<String, Vec<String>>>;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum IamPolicyEffect {
    #[default]
    Allow,
    Deny,
}

fn default_statement_effect() -> IamPolicyEffect {
    IamPolicyEffect::Allow
}
