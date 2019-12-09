use lambda::{lambda, LambdaCtx};
type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

fn main() {}

// #[lambda]
// #[tokio::main]
// async fn main(event: String, ctx: LambdaCtx) -> Result<String, Error> {
//     let _ = ctx;
//     Ok(event)
// }
