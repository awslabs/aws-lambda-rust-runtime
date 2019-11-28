use crate::requests::{IntoRequest, NextEventRequest};
use crate::Error;
use async_stream::try_stream;
use futures::prelude::*;
use http::{Request, Response, Uri};
use hyper::Body;

#[derive(Debug, Clone)]
pub(crate) struct Client {
    base: Uri,
    client: hyper::Client<hyper::client::HttpConnector>,
}

impl Client {
    pub fn new(base: Uri) -> Self {
        Self {
            base,
            client: hyper::Client::new(),
        }
    }

    fn set_origin(&self, req: Request<Vec<u8>>) -> Result<Request<Vec<u8>>, anyhow::Error> {
        let (mut parts, body) = req.into_parts();
        let (scheme, authority) = {
            let scheme = self
                .base
                .scheme_part()
                .expect("Scheme not found");
            let authority = self
                .base
                .authority_part()
                .expect("Authority not found");
            (scheme, authority)
        };
        let path = parts
            .uri
            .path_and_query()
            .expect("PathAndQuery not found");

        let uri = Uri::builder()
            .scheme(scheme.clone())
            .authority(authority.clone())
            .path_and_query(path.clone())
            .build()
            .expect("Unable to build URI");

        parts.uri = uri;
        Ok(Request::from_parts(parts, body))
    }

    pub(crate) async fn call(&mut self, req: Request<Vec<u8>>) -> Result<Response<Body>, anyhow::Error> {
        let req = self.set_origin(req)?;
        let (parts, body) = req.into_parts();
        let body = Body::from(body);
        let req = Request::from_parts(parts, body);
        let response = self.client.request(req).await.map_err(Error::Hyper)?;
        Ok(response)
    }
}

pub(crate) fn events(client: Client) -> impl Stream<Item = Result<Response<Body>, anyhow::Error>> {
    try_stream! {
        let mut client = client;
        loop {
            let req = NextEventRequest;
            let req = req.into_req()?;
            let event = client.call(req).await?;
            yield event;
        }
    }
}
