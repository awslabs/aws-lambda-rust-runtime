use lambda_extension::{extension_fn, Error, NextEvent};

async fn my_extension(event: NextEvent) -> Result<(), Error> {
    match event {
        NextEvent::Shutdown(_e) => {
            // do something with the shutdown event
        }
        NextEvent::Invoke(_e) => {
            // do something with the invoke event
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = extension_fn(my_extension);
    lambda_extension::run(func).await
}
