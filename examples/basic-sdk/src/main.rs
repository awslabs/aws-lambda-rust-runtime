use async_trait::async_trait;
use aws_sdk_s3::{operation::list_objects_v2::ListObjectsV2Output, Client as S3Client};
use lambda_runtime::{service_fn, tracing, Error, LambdaEvent};
use serde::{Deserialize, Serialize};

/// The request defines what bucket to list
#[derive(Deserialize)]
struct Request {
    bucket: String,
}

/// The response contains a Lambda-generated request ID and
/// the list of objects in the bucket.
#[derive(Serialize)]
struct Response {
    req_id: String,
    bucket: String,
    objects: Vec<String>,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
trait ListObjects {
    async fn list_objects(&self, bucket: &str) -> Result<ListObjectsV2Output, Error>;
}

#[async_trait]
impl ListObjects for S3Client {
    async fn list_objects(&self, bucket: &str) -> Result<ListObjectsV2Output, Error> {
        self.list_objects_v2().bucket(bucket).send().await.map_err(|e| e.into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    let shared_config = aws_config::from_env().load().await;
    let client = S3Client::new(&shared_config);
    let client_ref = &client;

    let func = service_fn(move |event| async move { my_handler(event, client_ref).await });
    lambda_runtime::run(func).await?;

    Ok(())
}

async fn my_handler<T: ListObjects>(event: LambdaEvent<Request>, client: &T) -> Result<Response, Error> {
    let bucket = event.payload.bucket;

    let objects_rsp = client.list_objects(&bucket).await?;
    let objects: Vec<_> = objects_rsp
        .contents()
        .into_iter()
        .filter_map(|o| o.key().map(|k| k.to_string()))
        .collect();

    // prepare the response
    let rsp = Response {
        req_id: event.context.request_id,
        bucket: bucket.clone(),
        objects,
    };

    // return `Response` (it will be serialized to JSON automatically by the runtime)
    Ok(rsp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_s3::types::Object;
    use lambda_runtime::{Context, LambdaEvent};
    use mockall::predicate::eq;

    #[tokio::test]
    async fn response_is_good_for_good_bucket() {
        let mut context = Context::default();
        context.request_id = "test-request-id".to_string();

        let mut mock_client = MockListObjects::default();
        mock_client
            .expect_list_objects()
            .with(eq("test-bucket"))
            .returning(|_| {
                Ok(ListObjectsV2Output::builder()
                    .contents(Object::builder().key("test-key-0").build())
                    .contents(Object::builder().key("test-key-1").build())
                    .contents(Object::builder().key("test-key-2").build())
                    .build())
            });

        let payload = Request {
            bucket: "test-bucket".to_string(),
        };
        let event = LambdaEvent { payload, context };

        let result = my_handler(event, &mock_client).await.unwrap();

        let expected_keys = vec![
            "test-key-0".to_string(),
            "test-key-1".to_string(),
            "test-key-2".to_string(),
        ];
        assert_eq!(result.req_id, "test-request-id".to_string());
        assert_eq!(result.bucket, "test-bucket".to_string());
        assert_eq!(result.objects, expected_keys);
    }

    #[tokio::test]
    async fn response_is_bad_for_bad_bucket() {
        let mut context = Context::default();
        context.request_id = "test-request-id".to_string();

        let mut mock_client = MockListObjects::default();
        mock_client
            .expect_list_objects()
            .with(eq("unknown-bucket"))
            .returning(|_| Err(Error::from("test-sdk-error")));

        let payload = Request {
            bucket: "unknown-bucket".to_string(),
        };
        let event = LambdaEvent { payload, context };

        let result = my_handler(event, &mock_client).await;
        assert!(result.is_err());
    }
}
