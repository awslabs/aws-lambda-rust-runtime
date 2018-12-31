use serde_derive;
use serde_derive::{Serialize, Deserialize};
use lambda_runtime;
use lambda_runtime::{lambda, Context, error::HandlerError};
use log;
use log::error;
use std::error::Error;
use std::collections;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use regex;
use regex::Regex;

#[derive(Serialize, Deserialize)]
struct CustomEvent {
    string: String,
}

#[derive(Serialize, Deserialize)]
struct CustomOutput {
    message: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    lambda!(my_handler);

    Ok(())
}

fn my_handler(event: CustomEvent, ctx: Context) -> Result<CustomOutput, HandlerError> {
    if event.string == "" {
        error!("Empty name in request {}", ctx.aws_request_id);
        return Err(ctx.new_error("Empty name"));
    }
    let mut map = collections::HashMap::<String, u32>::new();
    let re = Regex::new(r"\w+").unwrap();
    for caps in re.captures_iter(&event.string) {
        if let Some(cap) = caps.get(0) {
            let word = cap.as_str();
            match map.entry(word.to_string()) {
                Occupied(mut view) => { *view.get_mut() += 1; }
                Vacant(view) => { view.insert(1); }
            }
        }
    }

    // Serialise to a json string
    let j = serde_json::to_string(&map).unwrap();

    Ok(CustomOutput {
        message: format!("{}", j),
    })
}