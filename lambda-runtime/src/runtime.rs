use std::{error::Error, marker::PhantomData, result};

use lambda_runtime_client::RuntimeClient;
use serde;
use serde_json;
use tokio::prelude::future::{Future, IntoFuture, loop_fn, Loop};
use tokio::runtime::Runtime as TokioRuntime;

use crate::{
    context::Context,
    env::{ConfigProvider, EnvConfigProvider, FunctionSettings},
    error::{HandlerError, RuntimeError},
};
use tokio::runtime::TaskExecutor;

const MAX_RETRIES: i8 = 3;

/// Functions acting as a handler must conform to this type.
pub trait Handler<E, O> {
    /// Run the handler.
    fn run(&mut self, event: E, ctx: Context) -> Result<O, HandlerError>;
}

impl<F, E, O> Handler<E, O> for F
where
    F: FnMut(E, Context) -> Result<O, HandlerError>,
{
    fn run(&mut self, event: E, ctx: Context) -> Result<O, HandlerError> {
        (*self)(event, ctx)
    }
}

/// Creates a new runtime and begins polling for events using Lambda's Runtime APIs.
///
/// # Arguments
///
/// * `f` A function pointer that conforms to the `Handler` type.
///
/// # Panics
/// The function panics if the Lambda environment variables are not set.
pub fn start<E, O>(f: impl Handler<E, O> + 'static, runtime: Option<TokioRuntime>)
where
    E: serde::de::DeserializeOwned + 'static,
    O: serde::Serialize + 'static,
{
    let mut runtime = runtime.unwrap_or_else(|| TokioRuntime::new().expect("Failed to start tokio runtime"));
    let task_executor = runtime.executor();
    runtime.block_on(start_with_config(f, &EnvConfigProvider::new(), task_executor)).unwrap();
}

#[macro_export]
macro_rules! lambda {
    ($handler:ident) => {
        $crate::start($handler, None)
    };
    ($handler:ident, $runtime:expr) => {
        $crate::start($handler, Some($runtime))
    };
    ($handler:expr) => {
        $crate::start($handler, None)
    };
    ($handler:expr, $runtime:expr) => {
        $crate::start($handler, Some($runtime))
    };
}

/// Internal implementation of the start method that receives a config provider. This method
/// is used for unit tests with a mock provider. The provider data is used to construct the
/// `HttpRuntimeClient` which is then passed to the `start_with_runtime_client()` function.
///
/// # Arguments
///
/// * `f` A function pointer that conforms to the `Handler` type.
/// * `config` An implementation of the `ConfigProvider` trait with static lifetime.
///
/// # Panics
/// The function panics if the `ConfigProvider` returns an error from the `get_runtime_api_endpoint()`
/// or `get_function_settings()` methods. The panic forces AWS Lambda to terminate the environment
/// and spin up a new one for the next invocation.
pub(crate) fn start_with_config<E, O, C>(f: impl Handler<E, O>, config: &C, task_executor: TaskExecutor) -> impl Future<Item=(), Error=String>
where
    E: serde::de::DeserializeOwned + 'static,
    O: serde::Serialize,
    C: ConfigProvider,
{
    // if we cannot find the endpoint we panic, nothing else we can do.
    let endpoint: String;
    match config.get_runtime_api_endpoint() {
        Ok(value) => endpoint = value,
        Err(e) => {
            panic!("Could not find runtime API env var: {}", e);
        }
    }

    // if we can't get the settings from the environment variable
    // we also panic.
    let function_config: FunctionSettings;
    let settings = config.get_function_settings();
    match settings {
        Ok(env_settings) => function_config = env_settings,
        Err(e) => {
            panic!("Could not find runtime API env var: {}", e);
        }
    }

    match RuntimeClient::new(endpoint, task_executor) {
        Ok(client) => {
            start_with_runtime_client(f, function_config, client)
        }
        Err(e) => {
            panic!("Could not create runtime client SDK: {}", e);
        }
    }
}

/// Starts the rust runtime with the given Runtime API client.
///
/// # Arguments
///
/// * `f` A function pointer that conforms to the `Handler` type.
/// * `client` An implementation of the `lambda_runtime_client::RuntimeClient`
///            trait with a lifetime that matches that of the environment,
///            in this case expressed as `'env`.
///
/// # Panics
/// The function panics if we cannot instantiate a new `RustRuntime` object.
pub(crate) fn start_with_runtime_client<E, O>(
    f: impl Handler<E, O>,
    func_settings: FunctionSettings,
    client: RuntimeClient,
) -> impl Future<Item=(), Error=String> where
    E: serde::de::DeserializeOwned + 'static,
    O: serde::Serialize,
{
    let mut lambda_runtime: Runtime<_, E, O>;
    match Runtime::new(f, func_settings, MAX_RETRIES, client) {
        Ok(r) => lambda_runtime = r,
        Err(e) => {
            panic!("Error while starting runtime: {}", e);
        }
    }

    // start the infinite loop
    lambda_runtime.start()
}

/// Internal representation of the runtime object that polls for events and communicates
/// with the Runtime APIs
pub(super) struct Runtime<F, E, O> {
    runtime_client: RuntimeClient,
    handler: F,
    max_retries: i8,
    settings: FunctionSettings,
    _phan: PhantomData<(E, O)>,
}

// generic methods implementation
impl<F, E, O> Runtime<F, E, O> {
    /// Creates a new instance of the `Runtime` object populated with the environment
    /// settings.
    ///
    /// # Arguments
    ///
    /// * `f` A function pointer that conforms to the `Handler` type.
    /// * `retries` The maximum number of times we should retry calling the Runtime APIs
    ///             for recoverable errors while polling for new events.
    ///
    /// # Return
    /// A `Result` for the `Runtime` object or a `errors::RuntimeSerror`. The runtime
    /// fails the init if this function returns an error. If we cannot find the
    /// `AWS_LAMBDA_RUNTIME_API` variable in the environment the function panics.
    pub(super) fn new(
        f: F,
        config: FunctionSettings,
        retries: i8,
        client: RuntimeClient,
    ) -> result::Result<Self, RuntimeError> {
        debug!(
            "Creating new runtime with {} max retries for endpoint {}",
            retries,
            client.get_endpoint()
        );
        Ok(Runtime {
            runtime_client: client,
            settings: config,
            handler: f,
            max_retries: retries,
            _phan: PhantomData,
        })
    }
}

// implementation of methods that require the Event and Output types
// to be compatible with `serde`'s Deserialize/Serialize.
impl<F, E, O> Runtime<F, E, O>
where
    F: Handler<E, O>,
    E: serde::de::DeserializeOwned + 'static,
    O: serde::Serialize,
{
    /// Starts the main event loop and begin polling or new events. If one of the
    /// Runtime APIs returns an unrecoverable error this method calls the init failed
    /// API and then panics.
    fn start(&mut self) -> impl Future<Item=(), Error=String> {
        debug!("Beginning main event loop");
        loop {
            let (event, ctx) = self.get_next_event().wait().unwrap();
            let request_id = ctx.aws_request_id.clone();
            info!("Received new event with AWS request id: {}", request_id);
            let function_outcome = self.invoke(event, ctx);
            match function_outcome {
                Ok(response) => {
                    debug!(
                        "Function executed succesfully for {}, pushing response to Runtime API",
                        request_id
                    );
                    match serde_json::to_vec(&response) {
                        Ok(response_bytes) => {
                            match self.runtime_client.event_response(request_id.clone(), response_bytes).wait() {
                                Ok(_) => info!("Response for {} accepted by Runtime API", request_id),
                                // unrecoverable error while trying to communicate with the endpoint.
                                // we let the Lambda Runtime API know that we have died
                                Err(e) => {
                                    error!("Could not send response for {} to Runtime API: {}", request_id, e);
                                    if !e.recoverable {
                                        error!(
                                            "Error for {} is not recoverable, sending fail_init signal and panicking.",
                                            request_id
                                        );
                                        self.runtime_client.fail_init(&e);
                                        panic!("Could not send response");
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!(
                                "Could not marshal output object to Vec<u8> JSON represnetation for request {}: {}",
                                request_id, e
                            );
                            self.runtime_client
                                .fail_init(&RuntimeError::unrecoverable(e.description()));
                            panic!("Failed to marshal handler output, panic");
                        }
                    }
                }
                Err(e) => {
                    debug!("Handler returned an error for {}: {}", request_id, e);
                    debug!("Attempting to send error response to Runtime API for {}", request_id);
                    match self.runtime_client.event_error(request_id.clone(), &e).wait() {
                        Ok(_) => info!("Error response for {} accepted by Runtime API", request_id),
                        Err(e) => {
                            error!("Unable to send error response for {} to Runtime API: {}", request_id, e);
                            if !e.recoverable {
                                error!(
                                    "Error for {} is not recoverable, sending fail_init signal and panicking",
                                    request_id
                                );
                                self.runtime_client.fail_init(&e);
                                panic!("Could not send error response");
                            }
                        }
                    }
                }
            }
        }

        #[allow(unreachable_code)]
        Ok(()).into_future()
    }

    /// Invoke the handler function. This method is split out of the main loop to
    /// make it testable.
    pub(super) fn invoke(&mut self, e: E, ctx: Context) -> Result<O, HandlerError> {
        (&mut self.handler).run(e, ctx)
    }

    /// Attempts to get the next event from the Runtime APIs and keeps retrying
    /// unless the error throws is not recoverable.
    ///
    /// # Return
    /// A `Future` resolving to the next `Event` object to be processed.
    pub(super) fn get_next_event(&self) -> impl Future<Item=(E, Context), Error=String> {
        let max_retries = self.max_retries;
        let runtime_client = self.runtime_client.clone();
        let settings = self.settings.clone();
        loop_fn((0, None), move |(iteration, maybe_error): (i8, Option<RuntimeError>)| {
            if let Some(err) = maybe_error {
                if iteration > max_retries {
                    error!("Unrecoverable error while fetching next event: {}", err);
                    match err.request_id.clone() {
                        Some(req_id) => {
                            return Box::new(runtime_client
                                .event_error(req_id, &err)
                                .map_err(|e| format!("Could not send event error response: {}", e))
                                // these errors are not recoverable. Either we can't communicate with the runtime APIs
                                // or we cannot parse the event. panic to restart the environment.
                                .then(|_| Err("Could not retrieve next event".to_owned()))) as Box<dyn Future<Item=_, Error=_>>
                        }
                        None => {
                            runtime_client.fail_init(&err);
                            unreachable!();
                        }
                    }
                }
            }

            let settings = settings.clone();
            Box::new(runtime_client.next_event().then(move |result| {
                match result {
                    Ok((ev_data, invocation_ctx)) => {
                        let parse_result = serde_json::from_slice(&ev_data);
                        match parse_result {
                            Ok(ev) => {
                                let mut handler_ctx = Context::new(settings.clone());
                                handler_ctx.invoked_function_arn = invocation_ctx.invoked_function_arn;
                                handler_ctx.aws_request_id = invocation_ctx.aws_request_id;
                                handler_ctx.xray_trace_id = invocation_ctx.xray_trace_id;
                                handler_ctx.client_context = invocation_ctx.client_context;
                                handler_ctx.identity = invocation_ctx.identity;
                                handler_ctx.deadline = invocation_ctx.deadline;

                                Ok(Loop::Break((ev, handler_ctx)))
                            }
                            Err(e) => {
                                error!("Could not parse event to type: {}", e);
                                let mut runtime_err = RuntimeError::from(e);
                                runtime_err.request_id = Some(invocation_ctx.aws_request_id);
                                Ok(Loop::Continue((iteration + 1, Some(runtime_err))))
                            }
                        }
                    }
                    Err(e) => Ok(Loop::Continue((iteration + 1, Some(RuntimeError::from(e))))),
                }
            })) as Box<dyn Future<Item=_, Error=_>>
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{context, env};
    use lambda_runtime_client::RuntimeClient;

    #[test]
    fn runtime_invokes_handler() {
        let config: &dyn env::ConfigProvider = &env::tests::MockConfigProvider { error: false };
        let runtime = TokioRuntime::new().expect("Could not create tokio runtime");
        let client = RuntimeClient::new(
            config
                .get_runtime_api_endpoint()
                .expect("Could not get runtime endpoint"),
            runtime.executor(),
        )
        .expect("Could not initialize client");
        let handler = |_e: String, _c: context::Context| -> Result<String, HandlerError> { Ok("hello".to_string()) };
        let retries: i8 = 3;
        let runtime = Runtime::new(
            handler,
            config
                .get_function_settings()
                .expect("Could not load environment config"),
            retries,
            client,
        );
        assert_eq!(
            runtime.is_err(),
            false,
            "Runtime threw an unexpected error: {}",
            runtime.err().unwrap()
        );
        let output = runtime
            .unwrap()
            .invoke(String::from("test"), context::tests::test_context(10));
        assert_eq!(
            output.is_err(),
            false,
            "Handler threw an unexpected error: {}",
            output.err().unwrap()
        );
        let output_string = output.unwrap();
        assert_eq!(output_string, "hello", "Unexpected output message: {}", output_string);
    }
}
