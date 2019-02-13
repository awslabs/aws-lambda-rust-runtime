use envy;

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct Config {
    #[serde(rename = "AWS_LAMBDA_RUNTIME_API")]
    pub endpoint: String,
    #[serde(rename = "AWS_LAMBDA_FUNCTION_NAME")]
    pub function_name: String,
    #[serde(rename = "AWS_LAMBDA_FUNCTION_MEMORY_SIZE")]
    pub memory: i32,
    #[serde(rename = "AWS_LAMBDA_FUNCTION_VERSION")]
    pub version: String,
    #[serde(rename = "AWS_LAMBDA_LOG_STREAM_NAME")]
    pub log_stream: String,
    #[serde(rename = "AWS_LAMBDA_LOG_GROUP_NAME")]
    pub log_group: String,
}

impl Config {
    pub(crate) fn from_env() -> Self {
        Config {
            endpoint: std::env::var("AWS_LAMBDA_RUNTIME_API").unwrap(),
            function_name: std::env::var("AWS_LAMBDA_FUNCTION_NAME").unwrap(),
            ..Config::default()
        }
    }
}
