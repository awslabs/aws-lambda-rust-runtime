use crate::encodings::Base64Data;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KinesisAnalyticsOutputDeliveryEvent {
    #[serde(default)]
    pub invocation_id: Option<String>,
    #[serde(default)]
    pub application_arn: Option<String>,
    pub records: Vec<KinesisAnalyticsOutputDeliveryEventRecord>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KinesisAnalyticsOutputDeliveryEventRecord {
    #[serde(default)]
    pub record_id: Option<String>,
    pub data: Base64Data,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KinesisAnalyticsOutputDeliveryResponse {
    pub records: Vec<KinesisAnalyticsOutputDeliveryResponseRecord>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KinesisAnalyticsOutputDeliveryResponseRecord {
    #[serde(default)]
    pub record_id: Option<String>,
    /// possible values include Ok and DeliveryFailed
    #[serde(default)]
    pub result: Option<String>,
}
