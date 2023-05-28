use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IoTButtonEvent {
    #[serde(default)]
    pub serial_number: Option<String>,
    #[serde(default)]
    pub click_type: Option<String>,
    #[serde(default)]
    pub battery_voltage: Option<String>,
}

#[cfg(test)]
mod test {
    use super::*;

    use serde_json;

    #[test]
    #[cfg(feature = "iot_button")]
    fn example_iot_button_event() {
        let data = include_bytes!("../../fixtures/example-iot_button-event.json");
        let parsed: IoTButtonEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: IoTButtonEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
