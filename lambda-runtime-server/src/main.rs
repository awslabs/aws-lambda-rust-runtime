use failure::Error;
use http::{Request, StatusCode, header::name::typed_headers};
use serde::{Deserialize, Serialize};
use serde_json::json;
use warp::{self, reply, path, body::FullBody, Filter, Rejection, Reply};

type One<T> = (T,);


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
    typed_headers! {
        (LambdaRuntimeAwsRequestId, LAMBDA_RUNTIME_AWS_REQUEST_ID, "lambda-runtime-aws-request-id");
    }

    let routes = next()
        .or(ok_response())
        .or(err_response())
        .or(init_error());

    warp::serve(routes).run(([127, 0, 0, 1], 3030));
}

fn next() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("runtime" / "invocation" / "next")
        .and(plain_request())
        .map(|_| {
            let json = json!({"message": "hello, world!"});
            reply::json(&json)
        })
}

fn ok_response() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("runtime" / "invocation" / String / "response")
        .and(plain_request())
        .and(warp::post2())
        .map(|request_id: String, request: Request<FullBody>| {
            reply::reply()
        })
}

fn err_response() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("runtime" / "invocation" / String / "error")
        .and(json_request())
        .and(warp::post2())
        .map(|request_id: String, request: Request<String>| {
            reply::reply()
        })
}

fn init_error() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("runtime" / "init" / "error")
        .and(json_request())
        .and(warp::post2())
        .map(|_request: Request<String>| {
            reply::reply()
        })
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

fn plain_request() -> impl Filter<Extract = One<Request<FullBody>>, Error = Rejection> + Clone {
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

#[test]
fn test_init_error_endpoint() -> Result<(), Error> {
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
fn test_ok_response_endpoint() -> Result<(), Error> {
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
fn test_error_response_endpoint() -> Result<(), Error> {
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
fn test_next_endpoint() -> Result<(), Error> {
    let filter = next();
    let res = warp::test::request()
        .path("/runtime/invocation/next")
        .method("GET")
        .reply(&filter);

    println!("Response: {:?}", res);
    assert_eq!(res.status(), 200);

    Ok(())
}