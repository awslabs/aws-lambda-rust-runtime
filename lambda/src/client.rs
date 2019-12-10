use crate::Err;
use http::{Request, Response, Uri};
use hyper::Body;
use std::task;
use tower_service::Service;

#[derive(Debug, Clone)]
pub(crate) struct Client<S> {
    base: Uri,
    client: S,
}

impl<S> Client<S>
where
    S: Service<Request<Body>, Response = Response<Body>>,
    <S as Service<Request<Body>>>::Error: Into<Err> + Send + Sync + 'static + std::error::Error,
{
    pub fn new(base: Uri, client: S) -> Self {
        Self { base, client }
    }

    fn set_origin<B>(&self, req: Request<B>) -> Result<Request<B>, Err> {
        let (mut parts, body) = req.into_parts();
        let (scheme, authority) = {
            let scheme = self.base.scheme().expect("Scheme not found");
            let authority = self.base.authority().expect("Authority not found");
            (scheme, authority)
        };
        let path = parts.uri.path_and_query().expect("PathAndQuery not found");

        let uri = Uri::builder()
            .scheme(scheme.clone())
            .authority(authority.clone())
            .path_and_query(path.clone())
            .build()
            .expect("Unable to build URI");

        parts.uri = uri;
        Ok(Request::from_parts(parts, body))
    }

    pub(crate) async fn call(&mut self, req: Request<Body>) -> Result<Response<Body>, Err> {
        let req = self.set_origin(req)?;
        let (parts, body) = req.into_parts();
        let body = Body::from(body);
        let req = Request::from_parts(parts, body);
        let response = self.client.call(req).await?;
        Ok(response)
    }
}

struct SimulatedClient;

impl Service<Request<Body>> for SimulatedClient {
    type Response = Response<Body>;
    type Error = Err;
    type Future = futures::future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _: &mut task::Context<'_>) -> task::Poll<Result<(), Self::Error>> {
        task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let (headers, body) = req.into_parts();
        let path_and_query = headers
            .uri
            .path_and_query()
            .expect("missing path and query");

        match path_and_query.as_str() {
            "/runtime/invocation/next" => unimplemented!(),
            "/runtime/invocation/{}/response" => unimplemented!(),
            "/runtime/invocation/{}/error" => unimplemented!(),
            _ => unimplemented!(),
        }
    }
}
