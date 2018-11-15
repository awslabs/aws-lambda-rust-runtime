use std::env;

use error::RuntimeError;

/// The name of the environment variable in the Lambda execution
/// environment for the Runtime APIs endpoint. The valie of this
/// variable is read once as the runtime starts.
pub const RUNTIME_ENDPOINT_VAR: &str = "AWS_LAMBDA_RUNTIME_API";

/// Clone-able generic function settings object. The data is loaded
/// from environment variables during the init process. The data
/// for the object is cloned in the `Context` for each invocation.
#[derive(Clone)]
pub struct FunctionSettings {
    pub function_name: String,
    pub memory_size: i32,
    pub version: String,
    pub log_stream: String,
    pub log_group: String,
}

/// Trait used by the `RustRuntime` module to retrieve configuration information
/// about the environement. This is implemented by the `EnvConfigProvider` using
/// the environment variables. We also have a mock implementation for the unit tests
pub trait ConfigProvider {
    /// Loads the function settings such as name, arn, memory amount, version, etc.
    ///
    /// # Return
    /// A `Result` of `FunctionSettings` or a `RuntimeError`. The runtime
    /// fails the init process if this function returns an error.
    fn get_function_settings(&self) -> Result<FunctionSettings, RuntimeError>;

    /// Returns the endpoint (hostname:port) for the Runtime API endpoint
    fn get_runtime_api_endpoint(&self) -> Result<String, RuntimeError>;
}

/// Implementation of the `ConfigProvider` trait that reads the settings from
/// environment variables in the Lambda execution environment. This is the config
/// used by the `start()` method of this module.
pub struct EnvConfigProvider;

impl ConfigProvider for EnvConfigProvider {
    /// Loads the function settings from the Lambda environment variables:
    /// https://docs.aws.amazon.com/lambda/latest/dg/current-supported-versions.html
    fn get_function_settings(&self) -> Result<FunctionSettings, RuntimeError> {
        let function_name = env::var("AWS_LAMBDA_FUNCTION_NAME")?;
        let version = env::var("AWS_LAMBDA_FUNCTION_VERSION")?;
        let log_stream = env::var("AWS_LAMBDA_LOG_STREAM_NAME")?;
        let log_group = env::var("AWS_LAMBDA_LOG_GROUP_NAME")?;
        let memory_str = env::var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE")?;
        let parsed_memory_str = memory_str.parse::<i32>();
        let memory_size: i32;
        match parsed_memory_str {
            Ok(int_value) => memory_size = int_value,
            Err(_parse_err) => {
                error!(
                    "Memory value from environment is not i32: {}",
                    env::var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE").unwrap()
                );
                return Err(RuntimeError::unrecoverable(&format!(
                    "Could not parse memory value: {}",
                    memory_str
                )));
            }
        };

        Ok(FunctionSettings {
            function_name,
            memory_size,
            version,
            log_stream,
            log_group,
        })
    }

    /// Loads the endpoint from Lambda's default environment variable: AWS_LAMBDA_RUNTIME_API
    fn get_runtime_api_endpoint(&self) -> Result<String, RuntimeError> {
        let endpoint = env::var(RUNTIME_ENDPOINT_VAR)?;
        Ok(endpoint)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use env::*;
    use error;
    use std::{env, error::Error};

    pub(crate) struct MockConfigProvider {
        pub(crate) error: bool,
    }

    impl ConfigProvider for MockConfigProvider {
        fn get_function_settings(&self) -> Result<FunctionSettings, error::RuntimeError> {
            if self.error {
                return Err(error::RuntimeError::unrecoverable("Mock error"));
            }

            Ok(FunctionSettings {
                function_name: String::from("MockFunction"),
                memory_size: 128,
                version: String::from("$LATEST"),
                log_stream: String::from("LogStream"),
                log_group: String::from("LogGroup"),
            })
        }

        fn get_runtime_api_endpoint(&self) -> Result<String, error::RuntimeError> {
            if self.error {
                return Err(error::RuntimeError::unrecoverable("Mock error"));
            }

            Ok(String::from("http://localhost:8080"))
        }
    }

    fn set_endpoint_env_var() {
        env::set_var(RUNTIME_ENDPOINT_VAR, "localhost:8080");
    }

    fn set_lambda_env_vars() {
        env::set_var("AWS_LAMBDA_FUNCTION_NAME", "test_func");
        env::set_var("AWS_LAMBDA_FUNCTION_VERSION", "$LATEST");
        env::set_var("AWS_LAMBDA_LOG_STREAM_NAME", "LogStreamName");
        env::set_var("AWS_LAMBDA_LOG_GROUP_NAME", "LogGroup2");
        env::set_var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE", "128");
    }

    fn unset_env_vars() {
        env::remove_var(RUNTIME_ENDPOINT_VAR);
        env::remove_var("AWS_LAMBDA_FUNCTION_NAME");
        env::remove_var("AWS_LAMBDA_FUNCTION_VERSION");
        env::remove_var("AWS_LAMBDA_LOG_STREAM_NAME");
        env::remove_var("AWS_LAMBDA_LOG_GROUP_NAME");
        env::remove_var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE");
    }

    #[test]
    fn function_config_from_env_vars() {
        unset_env_vars();
        set_endpoint_env_var();
        set_lambda_env_vars();
        let config_provider: &ConfigProvider = &EnvConfigProvider {};
        let env_settings = config_provider.get_function_settings();
        assert_eq!(
            env_settings.is_err(),
            false,
            "Env settings returned an error: {}",
            env_settings.err().unwrap().description()
        );
        let settings = env_settings.unwrap();
        assert_eq!(
            settings.memory_size, 128,
            "Invalid memory size: {}",
            settings.memory_size
        );
        let endpoint = config_provider.get_runtime_api_endpoint();
        assert_eq!(
            endpoint.is_err(),
            false,
            "Env endpoint returned an error: {}",
            endpoint.err().unwrap().description()
        );

        unset_env_vars();
        let err_env_settings = config_provider.get_function_settings();
        assert!(
            err_env_settings.is_err(),
            "Env config did not return error without variables"
        );
        let err_endpoint = config_provider.get_runtime_api_endpoint();
        assert!(
            err_endpoint.is_err(),
            "Env endpoint did not return error without variables"
        );
    }
}
