use lambda_http::{crac, service_fn, Body, Error, IntoResponse, Request, RequestExt, Response, Runtime};
use std::sync::Arc;
use std::sync::RwLock;
use uuid::Uuid;

struct SharedClient {
    name: &'static str,
    secret: Arc<RwLock<String>>,
}

impl SharedClient {
    fn new(name: &'static str, secret: String) -> Self {
        Self {
            name,
            secret: Arc::new(RwLock::new(secret)),
        }
    }

    fn response(&self, req_id: String, first_name: &str) -> String {
        format!("{}: Client ({}) invoked by {}.", req_id, self.name, first_name)
    }
}

impl crac::Resource for SharedClient {
    fn before_checkpoint(&self) -> Result<(), Error> {
        // clear the secret before checkpointing
        {
            let mut write_lock = self.secret.write().unwrap();
            *write_lock = String::new();
        } // release the write lock

        {
            tracing::info!("in before_checkpoint: secret={:?}", self.secret.read().unwrap());
        } // release the read lock

        Ok(())
    }
    fn after_restore(&self) -> Result<(), Error> {
        // regenerate the secret after restoring
        {
            let mut write_lock = self.secret.write().unwrap();
            *write_lock = Uuid::new_v4().to_string();
        } // release the write lock

        {
            tracing::info!("in after_restore: secret={:?}", self.secret.read().unwrap());
        } // release the read lock

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    // Create the "client" and a reference to it, so that we can pass this into the handler closure below.
    let secret = Uuid::new_v4().to_string();
    let shared_client = SharedClient::new("random_client_name_1", secret);
    let shared_client_ref = &shared_client;
    {
        tracing::info!(
            "In main function: secret={:?}",
            shared_client_ref.secret.read().unwrap()
        );
    } // release the read lock

    // Define a closure here that makes use of the shared client.
    let handler_func_closure = move |event: Request| async move {
        {
            tracing::info!(
                "In handler function: secret={:?}",
                shared_client_ref.secret.read().unwrap()
            );
        } // release the read lock

        Result::<Response<Body>, Error>::Ok(
            match event
                .query_string_parameters_ref()
                .and_then(|params| params.first("first_name"))
            {
                Some(first_name) => {
                    shared_client_ref
                        .response(
                            event
                                .lambda_context_ref()
                                .map(|ctx| ctx.request_id.clone())
                                .unwrap_or_default(),
                            first_name,
                        )
                        .into_response()
                        .await
                }
                None => Response::builder()
                    .status(400)
                    .body("Empty first name".into())
                    .expect("failed to render response"),
            },
        )
    };

    // Pass the closure to the runtime here.
    Runtime::new()
        .register(shared_client_ref)
        .run(service_fn(handler_func_closure))
        .await?;
    Ok(())
}
