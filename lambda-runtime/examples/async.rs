use std::error::Error;

use lambda_runtime::{error::HandlerError, lambda, Context};
use serde_derive::{Deserialize, Serialize};
use simple_logger;
use tokio::prelude::future::{ok, Future};

#[derive(Deserialize)]
struct CustomEvent {
    #[serde(rename = "firstName")]
    first_name: String,
}

#[derive(Serialize)]
struct CustomOutput {
    message: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    lambda!(my_handler);

    Ok(())
}

fn my_handler(e: CustomEvent, _c: Context) -> impl Future<Item = CustomOutput, Error = HandlerError> {
    ok(format!("Hello, {}!", e.first_name))
        .map(|message| format!("{} (modified in a Future)", message))
        .map(|message| CustomOutput { message })
}
