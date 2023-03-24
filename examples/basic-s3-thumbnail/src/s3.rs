use async_trait::async_trait;
use aws_sdk_s3::{error::GetObjectError, types::ByteStream, Client as S3Client};

#[async_trait]
pub trait GetFile {
    async fn get_file(&self, bucket: &str, key: &str) -> Result<Vec<u8>, GetObjectError>;
}

#[async_trait]
pub trait PutFile {
    async fn put_file(&self, bucket: &str, key: &str, bytes: Vec<u8>) -> Result<String, String>;
}

#[async_trait]
impl GetFile for S3Client {
    async fn get_file(&self, bucket: &str, key: &str) -> Result<Vec<u8>, GetObjectError> {
        tracing::info!("get file bucket {}, key {}", bucket, key);

        let output = self.get_object().bucket(bucket).key(key).send().await;

        return match output {
            Ok(response) => {
                let bytes = response.body.collect().await.unwrap().to_vec();
                tracing::info!("Object is downloaded, size is {}", bytes.len());
                Ok(bytes)
            }
            Err(err) => {
                let service_err = err.into_service_error();
                let meta = service_err.meta();
                tracing::info!("Error from aws when downloding: {}", meta.to_string());
                Err(service_err)
            }
        };
    }
}

#[async_trait]
impl PutFile for S3Client {
    async fn put_file(&self, bucket: &str, key: &str, vec: Vec<u8>) -> Result<String, String> {
        tracing::info!("put file bucket {}, key {}", bucket, key);
        let bytes = ByteStream::from(vec);
        let result = self.put_object().bucket(bucket).key(key).body(bytes).send().await;

        match result {
            Ok(_) => Ok(format!("Uploaded a file with key {} into {}", key, bucket)),
            Err(err) => Err(err.into_service_error().meta().message().unwrap().to_string()),
        }
    }
}
