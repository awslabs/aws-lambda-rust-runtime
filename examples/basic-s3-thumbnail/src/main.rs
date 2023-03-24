use std::io::Cursor;

use aws_lambda_events::{event::s3::S3Event, s3::S3EventRecord};
use aws_sdk_s3::Client as S3Client;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use s3::{GetFile, PutFile};
use thumbnailer::{create_thumbnails, ThumbnailSize};

mod s3;

/**
This lambda handler
    * listen to file creation events
    * downloads the created file
    * creates a thumbnail from it
    * uploads the thumbnail to bucket "[original bucket name]-thumbs".

Make sure that
    * the created png file has no strange characters in the name
    * there is another bucket with "-thumbs" suffix in the name
    * this lambda only gets event from png file creation
    * this lambda has permission to put file into the "-thumbs" bucket
*/
pub(crate) async fn function_handler<T: PutFile + GetFile>(
    event: LambdaEvent<S3Event>,
    size: u32,
    client: &T,
) -> Result<(), Error> {
    let records = event.payload.records;

    for record in records.iter() {
        let (bucket, key) = get_file_props(record);

        if bucket.is_empty() || key.is_empty() {
            // The event is not a create event or bucket/object key is missing
            tracing::info!("record skipped");
            continue;
        }

        let reader = client.get_file(&bucket, &key).await;

        if reader.is_err() {
            continue;
        }

        let thumbnail = get_thumbnail(reader.unwrap(), size);

        let mut thumbs_bucket = bucket.to_owned();
        thumbs_bucket.push_str("-thumbs");

        // It uploads the thumbnail into a bucket name suffixed with "-thumbs"
        // So it needs file creation permission into that bucket

        let _ = client.put_file(&thumbs_bucket, &key, thumbnail).await;
    }

    Ok(())
}

fn get_file_props(record: &S3EventRecord) -> (String, String) {
    let empty_response = ("".to_string(), "".to_string());

    if record.event_name.is_none() {
        return empty_response;
    }

    if !record.event_name.as_ref().unwrap().starts_with("ObjectCreated") {
        return empty_response;
    }

    if record.s3.bucket.name.is_none() || record.s3.object.key.is_none() {
        return empty_response;
    }

    let bucket_name = record.s3.bucket.name.to_owned().unwrap();
    let object_key = record.s3.object.key.to_owned().unwrap();

    if bucket_name.is_empty() || object_key.is_empty() {
        tracing::info!("Bucket name or object_key is empty");
        return empty_response;
    }

    tracing::info!("Bucket: {}, Object key: {}", bucket_name, object_key);

    (bucket_name, object_key)
}

fn get_thumbnail(vec: Vec<u8>, size: u32) -> Vec<u8> {
    let reader = Cursor::new(vec);
    let mut thumbnails = create_thumbnails(reader, mime::IMAGE_PNG, [ThumbnailSize::Custom((size, size))]).unwrap();

    let thumbnail = thumbnails.pop().unwrap();
    let mut buf = Cursor::new(Vec::new());
    thumbnail.write_png(&mut buf).unwrap();

    buf.into_inner()
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // this needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let shared_config = aws_config::load_from_env().await;
    let client = S3Client::new(&shared_config);
    let client_ref = &client;

    let func = service_fn(move |event| async move { function_handler(event, 128, client_ref).await });

    run(func).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::BufReader;
    use std::io::Read;

    use super::*;
    use async_trait::async_trait;
    use aws_lambda_events::chrono::DateTime;
    use aws_lambda_events::s3::S3Bucket;
    use aws_lambda_events::s3::S3Entity;
    use aws_lambda_events::s3::S3Object;
    use aws_lambda_events::s3::S3RequestParameters;
    use aws_lambda_events::s3::S3UserIdentity;
    use aws_sdk_s3::error::GetObjectError;
    use lambda_runtime::{Context, LambdaEvent};
    use mockall::mock;
    use s3::GetFile;
    use s3::PutFile;

    #[tokio::test]
    async fn response_is_good() {
        let mut context = Context::default();
        context.request_id = "test-request-id".to_string();

        let bucket = "test-bucket";
        let key = "test-key";

        mock! {
            FakeS3Client {}

            #[async_trait]
            impl GetFile for FakeS3Client {
                pub async fn get_file(&self, bucket: &str, key: &str) -> Result<Vec<u8>, GetObjectError>;
            }
            #[async_trait]
            impl PutFile for FakeS3Client {
                pub async fn put_file(&self, bucket: &str, key: &str, bytes: Vec<u8>) -> Result<String, String>;
            }
        }

        let mut mock = MockFakeS3Client::new();

        mock.expect_get_file()
            .withf(|b: &str, k: &str| b.eq(bucket) && k.eq(key))
            .returning(|_1, _2| Ok(get_file("testdata/image.png")));

        mock.expect_put_file()
            .withf(|bu: &str, ke: &str, by| {
                let thumbnail = get_file("testdata/thumbnail.png");
                return bu.eq("test-bucket-thumbs") && ke.eq(key) && by == &thumbnail;
            })
            .returning(|_1, _2, _3| Ok("Done".to_string()));

        let payload = get_s3_event("ObjectCreated", bucket, key);
        let event = LambdaEvent { payload, context };

        let result = function_handler(event, 10, &mock).await.unwrap();

        assert_eq!((), result);
    }

    fn get_file(name: &str) -> Vec<u8> {
        let f = File::open(name);
        let mut reader = BufReader::new(f.unwrap());
        let mut buffer = Vec::new();

        reader.read_to_end(&mut buffer).unwrap();

        return buffer;
    }

    fn get_s3_event(event_name: &str, bucket_name: &str, object_key: &str) -> S3Event {
        return S3Event {
            records: (vec![get_s3_event_record(event_name, bucket_name, object_key)]),
        };
    }

    fn get_s3_event_record(event_name: &str, bucket_name: &str, object_key: &str) -> S3EventRecord {
        let s3_entity = S3Entity {
            schema_version: (Some(String::default())),
            configuration_id: (Some(String::default())),
            bucket: (S3Bucket {
                name: (Some(bucket_name.to_string())),
                owner_identity: (S3UserIdentity {
                    principal_id: (Some(String::default())),
                }),
                arn: (Some(String::default())),
            }),
            object: (S3Object {
                key: (Some(object_key.to_string())),
                size: (Some(1)),
                url_decoded_key: (Some(String::default())),
                version_id: (Some(String::default())),
                e_tag: (Some(String::default())),
                sequencer: (Some(String::default())),
            }),
        };

        return S3EventRecord {
            event_version: (Some(String::default())),
            event_source: (Some(String::default())),
            aws_region: (Some(String::default())),
            event_time: (DateTime::default()),
            event_name: (Some(event_name.to_string())),
            principal_id: (S3UserIdentity {
                principal_id: (Some("X".to_string())),
            }),
            request_parameters: (S3RequestParameters {
                source_ip_address: (Some(String::default())),
            }),
            response_elements: (HashMap::new()),
            s3: (s3_entity),
        };
    }
}
