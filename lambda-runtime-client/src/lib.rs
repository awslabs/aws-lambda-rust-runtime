#![warn(missing_docs)]
#![deny(warnings)]
#![allow(clippy::new_ret_no_self)]
//! Rust client SDK for the AWS Lambda Runtime APIs. This crate defines
//! a `RuntimeClient` that encapsulates interactions with AWS Lambda's Runtime
//! APIs.
//!
//! To return errors to the Runtime APIs through the `event_error()` or
//! `fail_init()` methods the `Error` objects must implement the `error::RuntimeApiError`
//! trait from this crate. The RuntimeApiError trait defines a single method
//! called `to_response()`. The method must return an `error::RuntimeError` object.
//! See the `error::ApiError` object in this crate for an example.
//!
//! # Examples
//!
//! ```rust,no_run
//! #[macro_use]
//! extern crate lambda_runtime_errors_derive;
//! extern crate lambda_runtime_client;
//! #[macro_use]
//! extern crate serde_derive;
//! extern crate serde_json;
//! extern crate tokio;
//! extern crate futures;
//!
//! use lambda_runtime_client::{RuntimeClient, EventContext};
//! use futures::future::{Future, Either};
//! use failure::Fail;
//!
//! #[derive(Serialize, Deserialize, Debug)]
//! struct CustomEvent {
//!     name: String,
//! }
//!
//! #[derive(Serialize, Deserialize, Debug)]
//! struct CustomResponse {
//!     surname: String,
//! }
//! 
//! #[derive(LambdaErrorExt, Fail, Debug)]
//! #[fail(display = "Input was invalid")]
//! struct CustomErrorType;
//! 
//! impl CustomErrorType {
//!     fn new(_: &str) -> Self {
//!         Self {}
//!     }
//! }
//!
//! fn main() {
//!     let client = RuntimeClient::new("http://localhost:8080", None)
//!         .expect("Could not initialize client");
//!
//!     tokio::run(client.next_event()
//!         .map_err(|_| panic!("Could not retrieve next event"))
//!         .and_then(move |(event_data, event_context)| {
//!             let custom_event: CustomEvent = serde_json::from_slice(&event_data)
//!                 .expect("Could not turn Vec<u8> into CustomEvent object");
//!
//!             println!("Event for {}", custom_event.name);
//!             if custom_event.name == "John" {
//!                 let resp_object = CustomResponse{ surname: String::from("Doe")};
//!                 let resp_vec = serde_json::to_vec(&resp_object)
//!                     .expect("Could not serialize CustomResponse to Vec<u8>");
//!                 Either::A(client.event_response(&event_context.aws_request_id, &resp_vec)
//!                     .map_err(|_| panic!("Response sent successfully")))
//!             } else {
//!                 // return a custom error by implementing the RuntimeApiError trait.
//!                 // See the error module for examples.
//!                 Either::B(client.event_error(&event_context.aws_request_id, &CustomErrorType::new("Invalid first name").into())
//!                     .map_err(|_| panic!("Could not send error response")))
//!             }
//!         }));
//! }
//! ```

mod client;
pub mod error;

pub use crate::client::*;
