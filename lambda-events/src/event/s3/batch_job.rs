/// `S3BatchJobEvent` encapsulates the detail of a s3 batch job
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3BatchJobEvent {
    #[serde(default)]
    pub invocation_schema_version: Option<String>,
    #[serde(default)]
    pub invocation_id: Option<String>,
    pub job: S3BatchJob,
    pub tasks: Vec<S3BatchJobTask>,
}

/// `S3BatchJob` whichs have the job id
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3BatchJob {
    #[serde(default)]
    pub id: Option<String>,
}

/// `S3BatchJobTask` represents one task in the s3 batch job and have all task details
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3BatchJobTask {
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub s3_key: Option<String>,
    #[serde(default)]
    pub s3_version_id: Option<String>,
    #[serde(default)]
    pub s3_bucket_arn: Option<String>,
}

/// `S3BatchJobResponse` is the response of a iven s3 batch job with the results
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3BatchJobResponse {
    #[serde(default)]
    pub invocation_schema_version: Option<String>,
    #[serde(default)]
    pub treat_missing_keys_as: Option<String>,
    #[serde(default)]
    pub invocation_id: Option<String>,
    pub results: Vec<S3BatchJobResult>,
}

/// `S3BatchJobResult` represents the result of a given task
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3BatchJobResult {
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub result_code: Option<String>,
    #[serde(default)]
    pub result_string: Option<String>,
}
