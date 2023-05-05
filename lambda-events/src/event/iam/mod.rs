/// `IamPolicyDocument` represents an IAM policy document.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IamPolicyDocument {
    #[serde(default)]
    #[serde(rename = "Version")]
    pub version: Option<String>,
    #[serde(rename = "Statement")]
    pub statement: Vec<IamPolicyStatement>,
}

/// `IamPolicyStatement` represents one statement from IAM policy with action, effect and resource.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IamPolicyStatement {
    #[serde(rename = "Action")]
    pub action: Vec<String>,
    #[serde(default)]
    #[serde(rename = "Effect")]
    pub effect: Option<String>,
    #[serde(rename = "Resource")]
    pub resource: Vec<String>,
}
