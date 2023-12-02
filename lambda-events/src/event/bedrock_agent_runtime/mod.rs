use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentEvent {
    ///The version of the message that identifies the format of the event data going into the Lambda function and the expected format of the response from a Lambda function. Amazon Bedrock only supports version 1.0.
    pub message_version: String,
    ///Contains information about the name, ID, alias, and version of the agent that the action group belongs to.
    pub agent: Agent,
    ///The user input for the conversation turn.
    pub input_text: String,
    /// The unique identifier of the agent session.
    pub session_id: String,
    /// The name of the action group.
    pub action_group: String,
    /// The path to the API operation, as defined in the OpenAPI schema.
    pub api_path: String,
    /// The method of the API operation, as defined in the OpenAPI schema.
    pub http_method: String,
    /// Contains a list of objects. Each object contains the name, type, and value of a parameter in the API operation, as defined in the OpenAPI schema.
    pub parameters: Vec<Parameter>,
    /// Contains the request body and its properties, as defined in the OpenAPI schema.
    pub request_body: RequestBody,
    /// Contains session attributes and their values.
    pub session_attributes: HashMap<String, String>,
    /// Contains prompt attributes and their values.
    pub prompt_session_attributes: HashMap<String, String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestBody {
    pub content: HashMap<String, Content>,
}


#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub properties: Vec<Property>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Property {
    pub name: String,
    pub r#type: String,
    pub value: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub name: String,
    pub r#type: String,
    pub value: String,
}


#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub name: String,
    pub id: String,
    pub alias: String,
    pub version: String,
}

#[cfg(test)]
mod tests {
    use serde_json;

    #[test]
    #[cfg(feature = "bedrock-agent-runtime")]
    fn example_bedrock_agent__runtime_event() {
        let data = include!("../../fixtures/example-bedrock-agent-runtime-event.json");
        let parsed: AgentEvent = serde_json::from_str(&data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: AgentEvent = serde_json::from_slice(&output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}