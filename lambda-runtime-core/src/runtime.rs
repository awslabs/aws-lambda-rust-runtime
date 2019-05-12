use crate::{
    context::Context,
    env::{ConfigProvider, EnvConfigProvider, FunctionSettings},
    error::RuntimeError,
    handler::Handler,
};
use failure::Fail;
use lambda_runtime_client::{error::ErrorResponse, RuntimeClient};
use lambda_runtime_errors::LambdaErrorExt;
use log::*;
use std::{fmt::Display, marker::PhantomData};
// use tokio::runtime::Runtime as TokioRuntime;
use tokio::prelude::future::{Future, Loop, Either, loop_fn, IntoFuture};

// include file generated during the build process
include!(concat!(env!("OUT_DIR"), "/metadata.rs"));

const MAX_RETRIES: i8 = 3;

/// Creates a new runtime and begins polling for events using Lambda's Runtime APIs.
///
/// # Arguments
///
/// * `f` A function pointer that conforms to the `Handler` type.
///
/// # Panics
/// The function panics if the Lambda environment variables are not set.
pub fn start<EventError, I>(f: impl Handler<EventError, I>/*, runtime: Option<TokioRuntime>*/) -> impl Future<Item=(), Error=()> + Send
where
    EventError: Fail + LambdaErrorExt + Display + Send + Sync,
    I: IntoFuture<Item=Vec<u8>, Error=EventError> + Send,
    I::Future: Send,
{
    start_with_config(f, &EnvConfigProvider::default()/*, runtime*/)
}

#[macro_export]
/// Starts an event listener which will parse incoming events into the even type requested by
/// `handler` and will invoke `handler` on each incoming event. Can optionally be passed a Tokio
/// `runtime` to build the listener on. If none is provided, it creates its own.
macro_rules! lambda {
    ($handler:ident) => {
        $crate::tokio::run($crate::start($handler))
    };
    ($handler:ident, $runtime:expr) => {
        $runtime.spawn($crate::start($handler))
    };
    ($handler:expr) => {
        $crate::tokio::run($crate::start($handler))
    };
    ($handler:expr, $runtime:expr) => {
        $runtime.spawn($crate::start($handler))
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
pub fn start_with_config<Config, EventError, I>(
    f: impl Handler<EventError, I>,
    config: &Config,
    // runtime: Option<TokioRuntime>,
) -> impl Future<Item=(), Error=()> + Send
where
    Config: ConfigProvider,
    EventError: Fail + LambdaErrorExt + Display + Send + Sync,
    I: IntoFuture<Item=Vec<u8>, Error=EventError> + Send,
    I::Future: Send,
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

    let info = Option::from(runtime_release().to_owned());

    match RuntimeClient::new(&endpoint, info/*, runtime*/) {
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
pub(crate) fn start_with_runtime_client<EventError, I>(
    f: impl Handler<EventError, I>,
    func_settings: FunctionSettings,
    client: RuntimeClient,
) -> impl Future<Item=(), Error=()> + Send
where
    EventError: Fail + LambdaErrorExt + Display + Send + Sync,
    I: IntoFuture<Item=Vec<u8>, Error=EventError> + Send,
    I::Future: Send,
{
    let lambda_runtime: Runtime<_, EventError, _> = Runtime::new(f, func_settings, MAX_RETRIES, client);

    // start the infinite loop
    lambda_runtime.start()
}

/// Internal representation of the runtime object that polls for events and communicates
/// with the Runtime APIs
pub(super) struct Runtime<Function, EventError, I> {
    runtime_client: RuntimeClient,
    handler: Function,
    max_retries: i8,
    settings: FunctionSettings,
    _phantom1: PhantomData<EventError>,
    _phantom2: PhantomData<I>,
}

// generic methods implementation
impl<Function, EventError, I> Runtime<Function, EventError, I>
where
    Function: Handler<EventError, I>,
    EventError: Fail + LambdaErrorExt + Display + Send + Sync,
    I: IntoFuture<Item=Vec<u8>, Error=EventError> + Send,
{
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
    pub(super) fn new(f: Function, config: FunctionSettings, retries: i8, client: RuntimeClient) -> Self {
        debug!(
            "Creating new runtime with {} max retries for endpoint {}",
            retries,
            client.get_endpoint()
        );

        Runtime {
            runtime_client: client,
            settings: config,
            handler: f,
            max_retries: retries,
            _phantom1: PhantomData,
            _phantom2: PhantomData,
        }
    }
}

// implementation of methods that require the Event and Output types
// to be compatible with `serde`'s Deserialize/Serialize.
impl<Function, EventError, I> Runtime<Function, EventError, I>
where
    Function: Handler<EventError, I>,
    EventError: Fail + LambdaErrorExt + Display + Send + Sync,
    I: IntoFuture<Item=Vec<u8>, Error=EventError> + Send,
    I::Future: Send,
{
    /// Starts the main event loop and begin polling or new events. If one of the
    /// Runtime APIs returns an unrecoverable error this method calls the init failed
    /// API and then panics.
    fn start(self) -> impl Future<Item=(), Error=()> + Send {
        debug!("Beginning main event loop");
        loop_fn(self, |runtime| {
            runtime.get_next_event(0, None).and_then(|(mut runtime, event, ctx)| {
                let request_id = ctx.aws_request_id.clone();
                info!("Received new event with AWS request id: {}", request_id);
                runtime.invoke(event, ctx).into_future().then(|function_outcome| match function_outcome {
                    Ok(response) => {
                        debug!(
                            "Function executed succesfully for {}, pushing response to Runtime API",
                            request_id
                        );
                        Either::A(runtime.runtime_client.event_response(&request_id, &response)
                            .then(move |r| {
                                match r {
                                    Ok(_) => info!("Response for {} accepted by Runtime API", request_id),
                                    // unrecoverable error while trying to communicate with the endpoint.
                                    // we let the Lambda Runtime API know that we have died
                                    Err(e) => {
                                        error!("Could not send response for {} to Runtime API: {}", request_id, e);
                                        if !e.is_recoverable() {
                                            error!(
                                                "Error for {} is not recoverable, sending fail_init signal and panicking.",
                                                request_id
                                            );
                                            runtime.runtime_client.fail_init(&ErrorResponse::from(e));
                                            panic!("Could not send response");
                                        }
                                    },
                                }
                                Ok(Loop::Continue(runtime))
                            }))
                    }
                    Err(e) => {
                        error!("Handler returned an error for {}: {}", request_id, e);
                        debug!("Attempting to send error response to Runtime API for {}", request_id);
                        Either::B(runtime.runtime_client.event_error(&request_id, &ErrorResponse::from(e))
                            .then(move |r| {
                                match r {
                                    Ok(_) => info!("Error response for {} accepted by Runtime API", request_id),
                                    Err(e) => {
                                        error!("Unable to send error response for {} to Runtime API: {}", request_id, e);
                                        if !e.is_recoverable() {
                                            error!(
                                                "Error for {} is not recoverable, sending fail_init signal and panicking",
                                                request_id
                                            );
                                            runtime.runtime_client.fail_init(&ErrorResponse::from(e));
                                            panic!("Could not send error response");
                                        }
                                    },
                                }
                                Ok(Loop::Continue(runtime))
                            }))
                    }
                })
            })
        })
    }

    /// Invoke the handler function. This method is split out of the main loop to
    /// make it testable.
    pub(super) fn invoke(&mut self, e: Vec<u8>, ctx: Context) -> I {
        (self.handler).run(e, ctx)
    }

    /// Attempts to get the next event from the Runtime APIs and keeps retrying
    /// unless the error throws is not recoverable.
    ///
    /// # Return
    /// The next `Event` object to be processed.
    pub(super) fn get_next_event(self, retries: i8, e: Option<RuntimeError>) -> impl Future<Item=(Self, Vec<u8>, Context), Error=()> + Send {
        loop_fn((self, retries, e), |(runtime, retries, e)| {
            if let Some(err) = e {
                if retries > runtime.max_retries {
                    error!("Unrecoverable error while fetching next event: {}", err);
                    let future = match err.request_id.clone() {
                        Some(req_id) => {
                            Either::A(runtime.runtime_client.event_error(&req_id, &ErrorResponse::from(err))
                                .map_err(|_| panic!("Could not send event error response")))
                        }
                        None => {
                            Either::B(runtime.runtime_client.fail_init(&ErrorResponse::from(err)))
                        }
                    };
                    // to avoid unreachable code
                    return Either::A(future.then(|_| Err(())).map_err(|_| {
                        // these errors are not recoverable. Either we can't communicate with the runtie APIs
                        // or we cannot parse the event. panic to restart the environment.
                        panic!("Could not retrieve next event");
                    }));
                }
            }

            Either::B(runtime.runtime_client.next_event()
                .then(move |r| {
                    match r {
                        Ok((ev_data, invocation_ctx)) => {
                            let mut handler_ctx = Context::new(runtime.settings.clone());
                            handler_ctx.invoked_function_arn = invocation_ctx.invoked_function_arn;
                            handler_ctx.aws_request_id = invocation_ctx.aws_request_id;
                            handler_ctx.xray_trace_id = invocation_ctx.xray_trace_id;
                            handler_ctx.client_context = invocation_ctx.client_context;
                            handler_ctx.identity = invocation_ctx.identity;
                            handler_ctx.deadline = invocation_ctx.deadline;

                            Ok(Loop::Break((runtime, ev_data, handler_ctx)))
                        },
                        Err(e) => Ok(Loop::Continue((runtime, retries + 1, Option::from(RuntimeError::from(e))))),
                    }
                }))
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{context, env};
    use lambda_runtime_client::RuntimeClient;
    use lambda_runtime_errors::HandlerError;

    #[test]
    fn runtime_invokes_handler() {
        let config: &dyn env::ConfigProvider = &env::tests::MockConfigProvider { error: false };
        let client = RuntimeClient::new(
            &config
                .get_runtime_api_endpoint()
                .expect("Could not get runtime endpoint"),
            None,
            // None,
        )
        .expect("Could not initialize client");
        let handler = |_e: Vec<u8>, _c: context::Context| -> Result<Vec<u8>, HandlerError> { Ok(b"hello".to_vec()) };
        let retries: i8 = 3;
        let mut runtime = Runtime::new(
            handler,
            config
                .get_function_settings()
                .expect("Could not load environment config"),
            retries,
            client,
        );
        let output = runtime.invoke(b"test".to_vec(), context::tests::test_context(10));
        assert_eq!(
            output.is_err(),
            false,
            "Handler threw an unexpected error: {}",
            output.err().unwrap()
        );
        let output_bytes = output.ok().unwrap();
        let output_string = String::from_utf8(output_bytes).unwrap();
        assert_eq!(output_string, "hello", "Unexpected output message: {}", output_string);
    }
}
