use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::{Client, Error as OtherError};
use lambda_http::{run, service_fn, Error, IntoResponse, Request};
use tracing::info;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Item {
    pub p_type: String,
    pub age: String,
    pub username: String,
    pub first: String,
    pub last: String,
}

impl Clone for Item {
    fn clone(&self) -> Item {
        return Item {
            p_type: self.p_type.clone(),
            age: self.age.clone(),
            username: self.username.clone(),
            first: self.first.clone(),
            last: self.last.clone(),
        };
    }
}
/// This is the main body for the function.
/// Write your code inside it.
/// You can see more examples in Runtime's repository:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
async fn function_handler(event: Request) -> Result<impl IntoResponse, Error> {
    // Extract some useful information from the request
    let body = event.body();
    let s = std::str::from_utf8(&body).expect("invalid utf-8 sequence");
    //Log into Cloudwatch
    info!(payload = %s, "JSON Payload received");

    //Serialize the data into the struct.
    let item: Item = serde_json::from_str(s).map_err(Box::new)?;

    //Setup the client to write to DynamoDB
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);

    //Insert into the table.
    add_item(&client, item.clone(), "lambda_dyno_example").await?;

    //Deserialize into json to return in the Response
    let j = serde_json::to_string(&item)?;

    Ok(j)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}

// Add an item to a table.
// snippet-start:[dynamodb.rust.add-item]
pub async fn add_item(client: &Client, item: Item, table: &str) -> Result<(), OtherError> {
    let user_av = AttributeValue::S(item.username);
    let type_av = AttributeValue::S(item.p_type);
    let age_av = AttributeValue::S(item.age);
    let first_av = AttributeValue::S(item.first);
    let last_av = AttributeValue::S(item.last);

    let request = client
        .put_item()
        .table_name(table)
        .item("username", user_av)
        .item("account_type", type_av)
        .item("age", age_av)
        .item("first_name", first_av)
        .item("last_name", last_av);

    info!("adding item to DynamoDB");

    let _resp = request.send().await?;

    Ok(())
}
