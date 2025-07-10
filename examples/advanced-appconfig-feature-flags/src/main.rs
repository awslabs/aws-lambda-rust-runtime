use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use std::env;

mod appconfig;
use crate::appconfig::{AppConfigClient, ConfigurationFetcher};

#[derive(Deserialize)]
struct Request {
    quote: String,
}

#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String,
}

#[derive(Deserialize)]
struct AppConfig {
    #[serde(rename = "spanish-response")]
    spanish_response: bool,
    // Add other fields as needed
}

async fn function_handler<T: ConfigurationFetcher + Send + Sync>(
    event: LambdaEvent<Request>,
    config_fetcher: &T,
) -> Result<Response, Error> {
    // Extract some useful info from the request
    let quote = event.payload.quote;

    // Send a GET request to the local AppConfig endpoint
    let config: AppConfig = config_fetcher.get_configuration().await?;

    // Use the feature flag
    let msg = if config.spanish_response {
        format!("{quote}, in spanish.")
    } else {
        format!("{quote}.")
    };

    // Return `Response` (it will be serialized to JSON automatically by the runtime)
    Ok(Response {
        req_id: event.context.request_id,
        msg,
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    // Extract the AppConfig port from the environment
    let app_config_port = env::var("AWS_APPCONFIG_EXTENSION_HTTP_PORT")
        .unwrap_or_else(|_| "2772".to_string())
        .parse::<u16>()
        .expect("Invalid port number for AWS_APPCONFIG_EXTENSION_HTTP_PORT");

    // Create a new AppConfigClient with the extracted port
    let app_config_client = AppConfigClient::new(
        &env::var("APPLICATION_ID").expect("APPLICATION_ID must be set"),
        &env::var("ENVIRONMENT_ID").expect("ENVIRONMENT_ID must be set"),
        &env::var("CONFIGURATION_PROFILE_ID").expect("CONFIGURATION_PROFILE_ID must be set"),
        app_config_port,
    );

    // Use a closure to capture app_config_client and pass it to function_handler
    run(service_fn(|event| function_handler(event, &app_config_client))).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use lambda_runtime::Context;
    use serde::de::DeserializeOwned;

    struct MockConfigFetcher {
        spanish_response: bool,
    }

    #[async_trait]
    impl ConfigurationFetcher for MockConfigFetcher {
        async fn get_configuration<T>(&self) -> Result<T, appconfig::AppConfigError>
        where
            T: DeserializeOwned + Send,
        {
            let value = serde_json::json!({
                "spanish-response": self.spanish_response
            });
            let value: T = serde_json::from_value(value)?;
            Ok(value)
        }
    }

    #[tokio::test]
    async fn test_function_handler_english() {
        let mock_fetcher = MockConfigFetcher {
            spanish_response: false,
        };
        let request = Request {
            quote: "Hello, world".to_string(),
        };
        let context = Context::default();
        let event = LambdaEvent::new(request, context);

        let result = function_handler(event, &mock_fetcher).await.unwrap();

        assert_eq!(result.msg, "Hello, world.");
    }

    #[tokio::test]
    async fn test_function_handler_spanish() {
        let mock_fetcher = MockConfigFetcher { spanish_response: true };
        let request = Request {
            quote: "Hello, world".to_string(),
        };
        let context = Context::default();
        let event = LambdaEvent::new(request, context);

        let result = function_handler(event, &mock_fetcher).await.unwrap();

        assert_eq!(result.msg, "Hello, world, in spanish.");
    }
}
