use std::io::Cursor;

use async_trait::async_trait;
use aws_sdk_s3::{types::ByteStream, Client as S3Client};
use thumbnailer::{create_thumbnails, ThumbnailSize};

#[async_trait]
pub trait GetFile {
    async fn get_file(&self, bucket: &str, key: &str) -> Option<Cursor<Vec<u8>>>;
}

#[async_trait]
pub trait PutFile {
    async fn put_file(&self, bucket: &str, key: &str, bytes: ByteStream) -> Result<String, String>;
}

pub trait GetThumbnail {
    fn get_thumbnail(&self, reader: Cursor<Vec<u8>>) -> ByteStream;
}

impl GetThumbnail for S3Client {
    fn get_thumbnail(&self, reader: Cursor<Vec<u8>>) -> ByteStream {
        let mut thumbnails = create_thumbnails(reader, mime::IMAGE_PNG, [ThumbnailSize::Small]).unwrap();

        let thumbnail = thumbnails.pop().unwrap();
        let mut buf = Cursor::new(Vec::new());
        thumbnail.write_png(&mut buf).unwrap();

        return ByteStream::from(buf.into_inner());
    }
}

#[async_trait]
impl GetFile for S3Client {
    async fn get_file(&self, bucket: &str, key: &str) -> Option<Cursor<Vec<u8>>> {
        println!("get file bucket {}, key {}", bucket, key);

        let output = self.get_object().bucket(bucket).key(key).send().await;

        let mut reader = None;

        if output.as_ref().ok().is_some() {
            let bytes = output.ok().unwrap().body.collect().await.unwrap().to_vec();
            println!("Object is downloaded, size is {}", bytes.len());
            reader = Some(Cursor::new(bytes));
        } else if output.as_ref().err().is_some() {
            let err = output.err().unwrap();
            let service_err = err.into_service_error();
            let meta = service_err.meta();
            println!("Error from aws when downloding: {}", meta.to_string());
        } else {
            println!("Unknown error when downloading");
        }

        return reader;
    }
}

#[async_trait]
impl PutFile for S3Client {
    async fn put_file(&self, bucket: &str, key: &str, bytes: ByteStream) -> Result<String, String> {
        println!("put file bucket {}, key {}", bucket, key);
        let result = self.put_object().bucket(bucket).key(key).body(bytes).send().await;

        if result.as_ref().is_ok() {
            return Ok(format!("Uploaded a file with key {} into {}", key, bucket));
        }

        return Err(result
            .err()
            .unwrap()
            .into_service_error()
            .meta()
            .message()
            .unwrap()
            .to_string());
    }
}
