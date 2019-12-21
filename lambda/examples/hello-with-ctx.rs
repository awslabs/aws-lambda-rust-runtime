use lambda::lambda;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[lambda]
#[tokio::main]
async fn main(event: String) -> Result<String, Error> {
    let ctx = lambda::context();
    dbg!(ctx);
    Ok(event)
}
