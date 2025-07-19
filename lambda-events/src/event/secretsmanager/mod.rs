use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SecretsManagerSecretRotationEvent {
    pub step: String,
    pub secret_id: String,
    pub client_request_token: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[cfg(feature = "secretsmanager")]
    fn example_secretsmanager_secret_rotation_event() {
        let data = include_bytes!("../../fixtures/example-secretsmanager-secret-rotation-event.json");
        let parsed: SecretsManagerSecretRotationEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: SecretsManagerSecretRotationEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
