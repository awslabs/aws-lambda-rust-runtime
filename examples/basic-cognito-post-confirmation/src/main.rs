use aws_config::BehaviorVersion;
use aws_lambda_events::event::cognito::CognitoEventUserPoolsPostConfirmation;
use aws_sdk_ses::{
    types::{Body, Content, Destination, Message},
    Client,
};
use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};

const SOURCE_EMAIL: &str = "<source_email>";

async fn function_handler(
    client: &aws_sdk_ses::Client,
    event: LambdaEvent<CognitoEventUserPoolsPostConfirmation>,
) -> Result<CognitoEventUserPoolsPostConfirmation, Error> {
    let payload = event.payload;

    if let Some(email) = payload.request.user_attributes.get("email") {
        let body = if let Some(name) = payload.request.user_attributes.get("name") {
            format!("Welcome {name}, you have been confirmed.")
        } else {
            "Welcome, you have been confirmed.".to_string()
        };
        send_post_confirmation_email(client, email, "Cognito Identity Provider registration completed", &body).await?;
    }

    // Cognito always expect a response with the same shape as
    // the event when it handles Post Confirmation triggers.
    Ok(payload)
}

async fn send_post_confirmation_email(client: &Client, email: &str, subject: &str, body: &str) -> Result<(), Error> {
    let destination = Destination::builder().to_addresses(email).build();
    let subject = Content::builder().data(subject).build()?;
    let body = Content::builder().data(body).build()?;

    let message = Message::builder()
        .body(Body::builder().text(body).build())
        .subject(subject)
        .build();

    client
        .send_email()
        .source(SOURCE_EMAIL)
        .destination(destination)
        .message(message)
        .send()
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = Client::new(&config);

    run(service_fn(|event| function_handler(&client, event))).await
}
