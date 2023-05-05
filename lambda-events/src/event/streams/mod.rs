/// `KinesisEventResponse` is the outer structure to report batch item failures for KinesisEvent.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KinesisEventResponse {
    pub batch_item_failures: Vec<KinesisBatchItemFailure>,
}

/// `KinesisBatchItemFailure` is the individual record which failed processing.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KinesisBatchItemFailure {
    #[serde(default)]
    pub item_identifier: Option<String>,
}

/// `DynamoDbEventResponse` is the outer structure to report batch item failures for DynamoDBEvent.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamoDbEventResponse {
    pub batch_item_failures: Vec<DynamoDbBatchItemFailure>,
}

/// `DynamoDbBatchItemFailure` is the individual record which failed processing.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamoDbBatchItemFailure {
    #[serde(default)]
    pub item_identifier: Option<String>,
}

/// `SqsEventResponse` is the outer structure to report batch item failures for SQSEvent.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SqsEventResponse {
    pub batch_item_failures: Vec<SqsBatchItemFailure>,
}

/// `SqsBatchItemFailure` is the individual record which failed processing.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SqsBatchItemFailure {
    #[serde(default)]
    pub item_identifier: Option<String>,
}
