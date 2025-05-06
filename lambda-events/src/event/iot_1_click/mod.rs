use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::custom_serde::deserialize_lambda_map;

/// `IoTOneClickEvent` represents a click event published by clicking button type
/// device.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IoTOneClickEvent {
    pub device_event: IoTOneClickDeviceEvent,
    pub device_info: IoTOneClickDeviceInfo,
    pub placement_info: IoTOneClickPlacementInfo,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IoTOneClickDeviceEvent {
    pub button_clicked: IoTOneClickButtonClicked,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IoTOneClickButtonClicked {
    #[serde(default)]
    pub click_type: Option<String>,
    #[serde(default)]
    pub reported_time: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IoTOneClickDeviceInfo {
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub attributes: HashMap<String, String>,
    #[serde(default)]
    pub type_: Option<String>,
    #[serde(default)]
    pub device_id: Option<String>,
    pub remaining_life: f64,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IoTOneClickPlacementInfo {
    #[serde(default)]
    pub project_name: Option<String>,
    #[serde(default)]
    pub placement_name: Option<String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub attributes: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub devices: HashMap<String, String>,
}

#[cfg(test)]
mod test {
    // To save on boiler plate, JSON data is parsed from a mut byte slice rather than an &str. The slice isn't actually mutated
    // when using serde-json, but it IS when using simd-json - so we also take care not to reuse the slice
    // once it has been deserialized.

    use super::*;

    #[test]
    #[cfg(feature = "iot_1_click")]
    fn example_iot_1_click_event() {
        let mut data = include_bytes!("../../fixtures/example-iot_1_click-event.json").to_vec();
        let parsed: IoTOneClickEvent = aws_lambda_json_impl::from_slice(data.as_mut_slice()).unwrap();
        let mut output = aws_lambda_json_impl::to_string(&parsed).unwrap().into_bytes();
        let reparsed: IoTOneClickEvent = aws_lambda_json_impl::from_slice(output.as_mut_slice()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
