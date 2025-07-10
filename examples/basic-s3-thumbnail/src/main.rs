use aws_lambda_events::{event::s3::S3Event, s3::S3EventRecord};
use aws_sdk_s3::Client as S3Client;
use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};
use s3::{GetFile, PutFile};

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

    for record in records.into_iter() {
        let (bucket, key) = match get_file_props(record) {
            Ok(touple) => touple,
            Err(msg) => {
                tracing::info!("Record skipped with reason: {}", msg);
                continue;
            }
        };

        let image = match client.get_file(&bucket, &key).await {
            Ok(vec) => vec,
            Err(msg) => {
                tracing::info!("Can not get file from S3: {}", msg);
                continue;
            }
        };

        let thumbnail = match get_thumbnail(image, size) {
            Ok(vec) => vec,
            Err(msg) => {
                tracing::info!("Can not create thumbnail: {}", msg);
                continue;
            }
        };

        let mut thumbs_bucket = bucket.to_owned();
        thumbs_bucket.push_str("-thumbs");

        // It uploads the thumbnail into a bucket name suffixed with "-thumbs"
        // So it needs file creation permission into that bucket

        match client.put_file(&thumbs_bucket, &key, thumbnail).await {
            Ok(msg) => tracing::info!(msg),
            Err(msg) => tracing::info!("Can not upload thumbnail: {}", msg),
        }
    }

    Ok(())
}

fn get_file_props(record: S3EventRecord) -> Result<(String, String), String> {
    record
        .event_name
        .filter(|s| s.starts_with("ObjectCreated"))
        .ok_or("Wrong event")?;

    let bucket = record
        .s3
        .bucket
        .name
        .filter(|s| !s.is_empty())
        .ok_or("No bucket name")?;

    let key = record.s3.object.key.filter(|s| !s.is_empty()).ok_or("No object key")?;

    Ok((bucket, key))
}

#[cfg(not(test))]
fn get_thumbnail(vec: Vec<u8>, size: u32) -> Result<Vec<u8>, String> {
    use std::io::Cursor;

    use thumbnailer::{create_thumbnails, ThumbnailSize};

    let reader = Cursor::new(vec);
    let mime = mime::IMAGE_PNG;
    let sizes = [ThumbnailSize::Custom((size, size))];

    let thumbnail = match create_thumbnails(reader, mime, sizes) {
        Ok(mut thumbnails) => thumbnails.pop().ok_or("No thumbnail created")?,
        Err(thumb_error) => return Err(thumb_error.to_string()),
    };

    let mut buf = Cursor::new(Vec::new());

    match thumbnail.write_png(&mut buf) {
        Ok(_) => Ok(buf.into_inner()),
        Err(_) => Err("Unknown error when Thumbnail::write_png".to_string()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    let shared_config = aws_config::load_from_env().await;
    let client = S3Client::new(&shared_config);
    let client_ref = &client;

    let func = service_fn(move |event| async move { function_handler(event, 128, client_ref).await });

    run(func).await?;

    Ok(())
}

#[cfg(test)]
fn get_thumbnail(vec: Vec<u8>, _size: u32) -> Result<Vec<u8>, String> {
    let s = unsafe { std::str::from_utf8_unchecked(&vec) };

    match s {
        "IMAGE" => Ok("THUMBNAIL".into()),
        _ => Err("Input is not IMAGE".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use async_trait::async_trait;
    //use aws_lambda_events::chrono::DateTime;
    use aws_lambda_events::s3::{S3Bucket, S3Entity, S3Object, S3RequestParameters, S3UserIdentity};
    use aws_sdk_s3::operation::get_object::GetObjectError;
    use lambda_runtime::{Context, LambdaEvent};
    use mockall::mock;
    use s3::{GetFile, PutFile};

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
                async fn get_file(&self, bucket: &str, key: &str) -> Result<Vec<u8>, GetObjectError>;
            }
            #[async_trait]
            impl PutFile for FakeS3Client {
                async fn put_file(&self, bucket: &str, key: &str, bytes: Vec<u8>) -> Result<String, String>;
            }
        }

        let mut mock = MockFakeS3Client::new();

        mock.expect_get_file()
            .withf(|b, k| b.eq(bucket) && k.eq(key))
            .returning(|_1, _2| Ok("IMAGE".into()));

        mock.expect_put_file()
            .withf(|bu, ke, by| {
                return bu.eq("test-bucket-thumbs") && ke.eq(key) && by.eq("THUMBNAIL".as_bytes());
            })
            .return_const(Ok("Done".to_string()));

        let payload = get_s3_event("ObjectCreated", bucket, key);
        let event = LambdaEvent { payload, context };

        let result = function_handler(event, 10, &mock).await.unwrap();

        assert_eq!((), result);
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
                owner_identity: Some(S3UserIdentity {
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
            event_time: (chrono::DateTime::default()),
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
