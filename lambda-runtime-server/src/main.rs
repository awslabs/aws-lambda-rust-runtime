use headers::{HeaderMap, HeaderMapExt, HeaderName};
use http::Request;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use warp::{self, body::FullBody, path, reply, Filter, Rejection, Reply};

pub mod types;

use crate::types::{AWSRequestId, FunctionArn, TraceId};

lazy_static! {
    pub static ref AWS_REQUEST_ID: HeaderName = HeaderName::from_static("lambda-runtime-aws-request-id");
    pub static ref FUNCTION_ARN: HeaderName = HeaderName::from_static("lambda-runtime-invoked-function-arn");
    pub static ref TRACE_ID: HeaderName = HeaderName::from_static("lambda-runtime-trace-id");
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
struct LambdaError {
    error_message: String,
    error_type: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
struct SampleEvent {
    error_message: String,
    error_type: String,
}

fn main() {
    let routes = next().or(ok_response()).or(err_response()).or(init_error());

    println!("{}", &AWS_REQUEST_ID.as_str());
    warp::serve(routes).run(([127, 0, 0, 1], 3030));
}

fn default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    let request_id = AWSRequestId(Uuid::new_v4().to_string());
    let function_arn = FunctionArn("an-arn".to_string());
    let trace_id = TraceId("Root=1-5bef4de7-ad49b0e87f6ef6c87fc2e700;".to_string());
    headers.typed_insert(request_id);
    headers.typed_insert(function_arn);
    headers.typed_insert(trace_id);
    headers
}

fn next() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("runtime" / "invocation" / "next")
        .and(plain_request())
        .map(|_| {
            let json = json!({"message": "hello, world!"});
            reply::json(&json)
        })
        .with(reply::with::headers(default_headers()))
}

fn ok_response() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("runtime" / "invocation" / String / "response")
        .and(plain_request())
        .and(warp::post2())
        .map(|_request_id: String, _request: Request<FullBody>| reply::reply())
}

fn err_response() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("runtime" / "invocation" / String / "error")
        .and(json_request())
        .and(warp::post2())
        .map(|_request_id: String, _request: Request<String>| reply::reply())
}

fn init_error() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("runtime" / "init" / "error")
        .and(json_request())
        .and(warp::post2())
        .map(|_request: Request<String>| reply::reply())
}

fn json_request() -> impl Filter<Extract = (Request<String>,), Error = Rejection> + Clone {
    warp::any()
        .and(warp::header::headers_cloned())
        .and(warp::body::json())
        .and(warp::method())
        .map(|headers, body, method| {
            let mut request: Request<String> = Request::new(body);
            *request.method_mut() = method;
            *request.headers_mut() = headers;
            request
        })
}

fn plain_request() -> impl Filter<Extract = (Request<FullBody>,), Error = Rejection> + Clone {
    warp::any()
        .and(warp::header::headers_cloned())
        .and(warp::body::concat())
        .and(warp::method())
        .map(|headers, body, method| {
            let mut request: Request<FullBody> = Request::new(body);
            *request.method_mut() = method;
            *request.headers_mut() = headers;
            request
        })
}

#[cfg(test)]
mod endpoints {
    use super::*;

    #[test]
    fn test_init_error() -> Result<(), failure::Error> {
        let filter = init_error();
        let lambda_error = LambdaError {
            error_message: "An error message".to_string(),
            error_type: "failure::Error".to_string(),
        };

        let res = warp::test::request()
            .path("/runtime/init/error")
            .method("POST")
            .json(&serde_json::to_string(&lambda_error)?)
            .reply(&filter);

        println!("{:?}", res);
        assert_eq!(res.status(), 200);

        Ok(())
    }

    #[test]
    fn test_ok_response() -> Result<(), failure::Error> {
        use uuid::Uuid;
        let request_id = Uuid::new_v4();

        let filter = ok_response();
        let res = warp::test::request()
            .path(&format!("/runtime/invocation/{}/response", request_id))
            .method("POST")
            .body("SUCCESS")
            .reply(&filter);

        println!("{:?}", res);
        assert_eq!(res.status(), 200);

        Ok(())
    }

    #[test]
    fn test_error_response() -> Result<(), failure::Error> {
        use uuid::Uuid;
        let request_id = Uuid::new_v4();

        let lambda_error = LambdaError {
            error_message: "An error message".to_string(),
            error_type: "failure::Error".to_string(),
        };

        let filter = err_response();
        let res = warp::test::request()
            .path(&format!("/runtime/invocation/{}/error", request_id))
            .method("POST")
            .json(&serde_json::to_string(&lambda_error)?)
            .reply(&filter);

        println!("{:?}", res);
        assert_eq!(res.status(), 200);

        Ok(())
    }

    #[test]
    fn test_next() -> Result<(), failure::Error> {
        let filter = next();
        let res = warp::test::request()
            .path("/runtime/invocation/next")
            .method("GET")
            .reply(&filter);

        println!("Response: {:?}", res);
        assert_eq!(res.status(), 200);
        assert!(res.headers().typed_get::<AWSRequestId>().is_some());
        assert!(res.headers().typed_get::<FunctionArn>().is_some());
        assert!(res.headers().typed_get::<TraceId>().is_some());

        Ok(())
    }
}
