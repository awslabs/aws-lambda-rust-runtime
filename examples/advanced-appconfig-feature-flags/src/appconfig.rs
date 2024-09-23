//! # Rust AppConfig Client
//!
//! This library provides a Rust client for interacting with the AWS AppConfig local extension for AWS Lambda and ECS.
//! It allows you to retrieve configuration data for your application based on the application ID, environment ID,
//! and configuration profile ID.
//!
//! ## Features
//!
//! - Simple API for retrieving configuration data
//! - Asynchronous operations using `tokio` and `reqwest`
//! - Error handling with custom `AppConfigError` type
//! - Deserialization of configuration data into user-defined types
//!
//! ## Usage
//!
//! ```rust
//! use appconfig::{AppConfigClient, ConfigurationFetcher, AppConfigError};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), AppConfigError> {
//!     let client = AppConfigClient::new("app_id", "env_id", "profile_id", 2772);
//!     
//!     let config: YourConfigType = client.get_configuration().await?;
//!     
//!     println!("Received config: {:?}", config);
//!     
//!     Ok(())
//! }
//! ```
use async_trait::async_trait;
use reqwest::Client;
use serde::de::DeserializeOwned;
use thiserror::Error;

#[derive(Clone)]
pub struct AppConfigClient {
    client: Client,
    application_id: String,
    environment_id: String,
    configuration_profile_id: String,
    port: u16,
}

impl AppConfigClient {
    pub fn new(application_id: &str, environment_id: &str, configuration_profile_id: &str, port: u16) -> Self {
        AppConfigClient {
            client: Client::new(),
            application_id: application_id.to_string(),
            environment_id: environment_id.to_string(),
            configuration_profile_id: configuration_profile_id.to_string(),
            port,
        }
    }
}

#[async_trait]
impl ConfigurationFetcher for AppConfigClient {
    async fn get_configuration<T>(&self) -> Result<T, AppConfigError>
    where
        T: DeserializeOwned + Send,
    {
        let url = format!(
            "http://localhost:{}/applications/{}/environments/{}/configurations/{}",
            self.port, self.application_id, self.environment_id, self.configuration_profile_id
        );

        let response = self.client.get(&url).send().await?;
        let config: T = response.json().await?;

        Ok(config)
    }
}

#[derive(Error, Debug)]
pub enum AppConfigError {
    #[error("Failed to send request: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),
}

#[async_trait]
pub trait ConfigurationFetcher {
    async fn get_configuration<T>(&self) -> Result<T, AppConfigError>
    where
        T: DeserializeOwned + Send;
}
