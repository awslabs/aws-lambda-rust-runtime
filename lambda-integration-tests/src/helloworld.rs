use aws_lambda_events::{
    apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse},
    http::HeaderMap,
};
use lambda_runtime::{service_fn, tracing, Error, LambdaEvent};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    let func = service_fn(func);
    lambda_runtime::spawn_graceful_shutdown_handler(|| async move {});
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(_event: LambdaEvent<ApiGatewayProxyRequest>) -> Result<ApiGatewayProxyResponse, Error> {
    let mut headers = HeaderMap::new();
    headers.insert("content-type", "text/html".parse().unwrap());
    let resp = ApiGatewayProxyResponse {
        status_code: 200,
        multi_value_headers: headers.clone(),
        is_base64_encoded: false,
        body: Some("Hello world!".into()),
        headers,
    };
    Ok(resp)
}
