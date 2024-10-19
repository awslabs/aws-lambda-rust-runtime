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

    #[test]
    #[cfg(feature = "iot_button")]
    fn example_iot_button_event() {
        let mut data = include_bytes!("../../fixtures/example-iot_button-event.json").to_vec();
        let parsed: IoTButtonEvent = aws_lambda_json_impl::from_slice(data.as_mut_slice()).unwrap();
        let mut output = aws_lambda_json_impl::to_string(&parsed).unwrap().into_bytes();
        let reparsed: IoTButtonEvent = aws_lambda_json_impl::from_slice(output.as_mut_slice()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
