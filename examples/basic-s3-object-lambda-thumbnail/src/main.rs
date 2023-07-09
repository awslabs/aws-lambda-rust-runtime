use std::{error, io::Cursor};

use aws_lambda_events::s3::object_lambda::{GetObjectContext, S3ObjectLambdaEvent};
use aws_sdk_s3::Client as S3Client;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use s3::{GetFile, SendFile};
use thumbnailer::{create_thumbnails, ThumbnailSize};

mod s3;

/**
This s3 object lambda handler
    * downloads the asked file
    * creates a PNG thumbnail from it
    * forwards it to the browser
*/
pub(crate) async fn function_handler<T: SendFile + GetFile>(
    event: LambdaEvent<S3ObjectLambdaEvent>,
    size: u32,
    client: &T,
) -> Result<String, Box<dyn error::Error>> {
    tracing::info!("handler starts");

    let context: GetObjectContext = event.payload.get_object_context.unwrap();

    let route = context.output_route;
    let token = context.output_token;
    let s3_url = context.input_s3_url;

    tracing::info!("Route: {}, s3_url: {}", route, s3_url);

    let image = client.get_file(s3_url)?;
    tracing::info!("Image loaded. Length: {}", image.len());

    let thumbnail = get_thumbnail(image, size);
    tracing::info!("thumbnail created. Length: {}", thumbnail.len());

    // It sends the thumbnail back to the user

    client.send_file(route, token, thumbnail).await

    /*
    match client.send_file(route, token, thumbnail).await {
        Ok(msg) => tracing::info!(msg),
        Err(msg) => tracing::info!(msg)
    };

    tracing::info!("handler ends");

    Ok(())
    */
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
        .with_max_level(tracing::Level::TRACE)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let shared_config = aws_config::load_from_env().await;
    let client = S3Client::new(&shared_config);
    let client_ref = &client;

    let func = service_fn(move |event| async move { function_handler(event, 128, client_ref).await });

    let _ = run(func).await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::BufReader;
    use std::io::Read;

    use super::*;
    use async_trait::async_trait;
    use aws_lambda_events::s3::object_lambda::Configuration;
    use aws_lambda_events::s3::object_lambda::HeadObjectContext;
    use aws_lambda_events::s3::object_lambda::ListObjectsContext;
    use aws_lambda_events::s3::object_lambda::ListObjectsV2Context;
    use aws_lambda_events::s3::object_lambda::UserIdentity;
    use aws_lambda_events::s3::object_lambda::UserRequest;
    use aws_lambda_events::serde_json::json;
    use lambda_runtime::{Context, LambdaEvent};
    use mockall::mock;
    use s3::GetFile;
    use s3::SendFile;

    #[tokio::test]
    async fn response_is_good() {
        mock! {
            FakeS3Client {}

            #[async_trait]
            impl GetFile for FakeS3Client {
                pub fn get_file(&self, url: String) -> Result<Vec<u8>, Box<dyn error::Error>>;
            }
            #[async_trait]
            impl SendFile for FakeS3Client {
                pub async fn send_file(&self, route: String, token: String, vec: Vec<u8>) -> Result<String, Box<dyn error::Error>>;
            }
        }

        let mut mock = MockFakeS3Client::new();

        mock.expect_get_file()
            .withf(|u: &String| u.eq("S3_URL"))
            .returning(|_1| Ok(get_file("testdata/image.png")));

        mock.expect_send_file()
            .withf(|r: &String, t: &String, by| {
                let thumbnail = get_file("testdata/thumbnail.png");
                return r.eq("O_ROUTE") && t.eq("O_TOKEN") && by == &thumbnail;
            })
            .returning(|_1, _2, _3| Ok("File sent.".to_string()));

        let payload = get_s3_event();
        let context = Context::default();
        let event = LambdaEvent { payload, context };

        let result = function_handler(event, 10, &mock).await.unwrap();

        assert_eq!(("File sent."), result);
    }

    fn get_file(name: &str) -> Vec<u8> {
        let f = File::open(name);
        let mut reader = BufReader::new(f.unwrap());
        let mut buffer = Vec::new();

        reader.read_to_end(&mut buffer).unwrap();

        return buffer;
    }

    fn get_s3_event() -> S3ObjectLambdaEvent {
        return S3ObjectLambdaEvent {
            x_amz_request_id: ("ID".to_string()),
            head_object_context: (Some(HeadObjectContext::default())),
            list_objects_context: (Some(ListObjectsContext::default())),
            get_object_context: (Some(GetObjectContext {
                input_s3_url: ("S3_URL".to_string()),
                output_route: ("O_ROUTE".to_string()),
                output_token: ("O_TOKEN".to_string()),
            })),
            list_objects_v2_context: (Some(ListObjectsV2Context::default())),
            protocol_version: ("VERSION".to_string()),
            user_identity: (UserIdentity::default()),
            user_request: (UserRequest::default()),
            configuration: (Configuration {
                access_point_arn: ("APRN".to_string()),
                supporting_access_point_arn: ("SAPRN".to_string()),
                payload: (json!(null)),
            }),
        };
    }
}
