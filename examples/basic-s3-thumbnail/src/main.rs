use aws_config::meta::region::RegionProviderChain;
use aws_lambda_events::{event::s3::S3Event, s3::S3EventRecord};
use aws_sdk_s3::Client as S3Client;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use s3client::{GetFile, GetThumbnail, PutFile};

mod s3client;

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
pub(crate) async fn function_handler<T: PutFile + GetFile + GetThumbnail>(
    event: LambdaEvent<S3Event>,
    client: &T,
) -> Result<String, String> {
    let result = Ok("".to_string());
    let records = event.payload.records;
    for record in records.iter() {
        let (bucket, key) = get_file_props(record);
        if bucket.is_empty() || key.is_empty() {
            // The event is not a create event or bucket/object key is missing
            println!("record skipped");
            continue;
        }

        let reader = client.get_file(&bucket, &key).await;

        if reader.is_none() {
            continue;
        }

        let thumbnail = client.get_thumbnail(reader.unwrap());

        let mut thumbs_bucket = bucket.to_owned();
        thumbs_bucket.push_str("-thumbs");

        // It uplaods the thumbnail into a bucket name suffixed with "-thumbs"
        // So it needs file creation permission into that bucket

        return client.put_file(&thumbs_bucket, &key, thumbnail).await;
    }

    return result;
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
        println!("Bucket name or object_key is empty");
        return empty_response;
    }

    println!("Bucket: {}, Object key: {}", bucket_name, object_key);

    return (bucket_name, object_key);
}

async fn get_client() -> S3Client {
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-2");
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = S3Client::new(&config);

    println!("client region {}", client.conf().region().unwrap().to_string());

    return client;
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

    let client = get_client().await;
    let client_ref = &client;

    let func = service_fn(move |event| async move { function_handler(event, client_ref).await });

    run(func).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::io::Cursor;

    use super::*;
    use async_trait::async_trait;
    use aws_lambda_events::chrono::DateTime;
    use aws_lambda_events::s3::S3Bucket;
    use aws_lambda_events::s3::S3Entity;
    use aws_lambda_events::s3::S3Object;
    use aws_lambda_events::s3::S3RequestParameters;
    use aws_lambda_events::s3::S3UserIdentity;
    use aws_sdk_s3::types::ByteStream;
    use lambda_runtime::{Context, LambdaEvent};
    use mockall::mock;
    use mockall::predicate::eq;
    use s3client::GetFile;
    use s3client::PutFile;

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
                pub async fn get_file(&self, bucket: &str, key: &str) -> Option<Cursor<Vec<u8>>>;
            }
            #[async_trait]
            impl PutFile for FakeS3Client {
                pub async fn put_file(&self, bucket: &str, key: &str, bytes: ByteStream) -> Result<String, String>;
            }

            impl GetThumbnail for FakeS3Client {
                fn get_thumbnail(&self, reader: Cursor<Vec<u8>>) -> ByteStream;
            }
        }

        let mut mock = MockFakeS3Client::new();

        mock.expect_get_file()
            .withf(|b: &str, k: &str| b.eq(bucket) && k.eq(key))
            .returning(|_1, _2| Some(Cursor::new(b"IMAGE".to_vec())));

        mock.expect_get_thumbnail()
            .with(eq(Cursor::new(b"IMAGE".to_vec())))
            .returning(|_| ByteStream::from_static(b"THUMBNAIL"));

        mock.expect_put_file()
            .withf(|bu: &str, ke: &str, _by| bu.eq("test-bucket-thumbs") && ke.eq(key))
            .returning(|_1, _2, _3| Ok("Done".to_string()));

        let payload = get_s3_event("ObjectCreated", bucket, key);
        let event = LambdaEvent { payload, context };

        let result = function_handler(event, &mock).await.unwrap();

        assert_eq!("Done", result);
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
