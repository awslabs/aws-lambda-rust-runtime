use bytes::{Bytes, IntoBuf, buf::FromBuf};
use failure::Fail;
use futures::{future::{result, FutureResult}, Async, Future, Poll, Stream};
use http::Uri;
use hyper::Client;
use serde_json::Value;
use std::fmt::{Debug, Display};
use tower_service::Service;

#[derive(Fail, Debug)]
enum RuntimeError {
    #[fail(display = "{}", _0)]
    Http(#[fail(cause)] hyper::error::Error),
    #[fail(display = "{}", _0)]
    Json(#[fail(cause)] serde_json::error::Error),
    #[fail(display = "{}", _0)]
    Utf8Error(#[fail(cause)] std::string::FromUtf8Error),
}

trait Handler<Event, Response, Error>
where
    Event: FromBuf,
    Response: IntoBuf,
    Error: Fail + Display + Debug + Sync + 'static,
{
    fn run(&mut self, event: Event) -> Result<Response, Error>;
}

impl<Event, Response, Error, F> Handler<Event, Response, Error> for F
where
    Event: FromBuf,
    Response: IntoBuf,
    Error: Fail + Display + Debug + Sync + 'static,
    F: FnMut(Event) -> Result<Response, Error>,
{
    fn run(&mut self, event: Event) -> Result<Response, Error> {
        (self)(event)
    }
}

impl<Event, Response, Error> Service<Event> for Handler<Event, Response, Error>
where
    Event: FromBuf,
    Response: IntoBuf,
    Error: Fail + Display + Debug + Sync,
{
    type Response = Response;
    type Error = Error;
    type Future = FutureResult<Self::Response, Self::Error>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        Ok(Async::Ready(()))
    }

    fn call(&mut self, req: Event) -> Self::Future {
        result(self.run(req))
    }
}

fn main() {
    let handler = |event: Bytes| -> Result<Bytes, RuntimeError> {
        let event = String::from_utf8(event.to_vec()).map_err(RuntimeError::Utf8Error)?;
        let value: Value = serde_json::from_str(&event).map_err(RuntimeError::Json)?;
        println!("{:#?}", value);
        Ok(Bytes::from(event))
    };

    // hydrate -> process -> terminate
    let f = hydrate() // Fetch the event from the runtime...
        .and_then(move |event| process(event, handler)) // ...run the user-provided handler..
        .then(complete) // ...and then, report on the status/failure of the Lambda invocation.
        .map_err(|error| eprintln!("{:?}", error));

    tokio::run(f);
}

fn hydrate() -> impl Future<Item = Bytes, Error = RuntimeError> {
    let client = Client::new();
    let uri = "http://httpbin.org/json".parse::<Uri>().unwrap();

    client
        .get(uri)
        .and_then(|res| res.into_body().concat2())
        .and_then(|body| Ok(body.into_bytes()))
        .map_err(RuntimeError::Http)
}

fn process<Event, Response, Error, Handler>(
    event: Event,
    mut handler: Handler,
) -> Result<Response, Error>
where
    Event: FromBuf,
    Response: IntoBuf,
    Error: Fail + Display + Debug + Sync + 'static,
    Handler: FnMut(Event) -> Result<Response, Error>,
{
    (handler)(event)
}

fn complete(outcome: Result<Bytes, RuntimeError>) -> Result<(), RuntimeError> {
    match outcome {
        Ok(_request_id) => println!("Done!"),
        Err(e) => eprintln!("Lambda function panicked: {}", e),
    }
    Ok(())
}
