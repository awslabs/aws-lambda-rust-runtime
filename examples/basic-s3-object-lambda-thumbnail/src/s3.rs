use async_trait::async_trait;
use aws_sdk_s3::{
    error::SdkError, operation::write_get_object_response::WriteGetObjectResponseError, primitives::ByteStream,
    Client as S3Client,
};
use lambda_runtime::tracing;
use std::{error, io::Read};

pub trait GetFile {
    fn get_file(&self, url: String) -> Result<Vec<u8>, Box<dyn error::Error>>;
}

#[async_trait]
pub trait SendFile {
    async fn send_file(&self, route: String, token: String, vec: Vec<u8>) -> Result<String, Box<dyn error::Error>>;
}

impl GetFile for S3Client {
    fn get_file(&self, url: String) -> Result<Vec<u8>, Box<dyn error::Error>> {
        tracing::info!("get file url {}", url);

        let mut res = ureq::get(&url).call()?;
        let len: usize = res
            .headers()
            .get("Content-Length")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse().ok())
            .unwrap();

        let mut bytes: Vec<u8> = Vec::with_capacity(len);

        std::io::Read::take(res.body_mut().as_reader(), 10_000_000).read_to_end(&mut bytes)?;

        tracing::info!("got {} bytes", bytes.len());

        Ok(bytes)
    }
}

#[async_trait]
impl SendFile for S3Client {
    async fn send_file(&self, route: String, token: String, vec: Vec<u8>) -> Result<String, Box<dyn error::Error>> {
        tracing::info!("send file route {}, token {}, length {}", route, token, vec.len());

        let bytes = ByteStream::from(vec);

        let write = self
            .write_get_object_response()
            .request_route(route)
            .request_token(token)
            .status_code(200)
            .body(bytes)
            .send()
            .await;

        if let Err(err) = write {
            check_error(err);
            Err("WriteGetObjectResponse creation error".into())
        } else {
            Ok("File sent.".to_string())
        }
    }
}

fn check_error(error: SdkError<WriteGetObjectResponseError>) {
    match error {
        SdkError::ConstructionFailure(_err) => {
            tracing::info!("ConstructionFailure");
        }
        SdkError::DispatchFailure(err) => {
            tracing::info!("DispatchFailure");
            if err.is_io() {
                tracing::info!("IO error");
            }
            if err.is_timeout() {
                tracing::info!("Timeout error");
            }
            if err.is_user() {
                tracing::info!("User error");
            }
            if err.is_other() {
                tracing::info!("Other error");
            }
        }
        SdkError::ResponseError(_err) => tracing::info!("ResponseError"),
        SdkError::TimeoutError(_err) => tracing::info!("TimeoutError"),
        SdkError::ServiceError(err) => {
            tracing::info!("ServiceError");
            let wgore = err.into_err();
            let meta = wgore.meta();
            let code = meta.code().unwrap_or_default();
            let msg = meta.message().unwrap_or_default();
            tracing::info!("code: {}, message: {}, meta: {}", code, msg, meta);
        }
        _ => tracing::info!("other error"),
    }
}
