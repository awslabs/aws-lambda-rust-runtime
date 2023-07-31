use diesel::{ConnectionError, ConnectionResult};
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use std::time::Duration;

use axum::{
    extract::{Path, State},
    response::Json,
    routing::get,
    Router,
};
use bb8::Pool;
use diesel::prelude::*;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection, RunQueryDsl};
use lambda_http::{http::StatusCode, run, Error};
use serde::{Deserialize, Serialize};

table! {
    posts (id) {
        id -> Integer,
        title -> Text,
        content -> Text,
        published -> Bool,
    }
}

#[derive(Default, Queryable, Selectable, Serialize)]
struct Post {
    id: i32,
    title: String,
    content: String,
    published: bool,
}

#[derive(Deserialize, Insertable)]
#[diesel(table_name = posts)]
struct NewPost {
    title: String,
    content: String,
    published: bool,
}

type AsyncPool = Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
type ServerError = (StatusCode, String);

async fn create_post(State(pool): State<AsyncPool>, Json(post): Json<NewPost>) -> Result<Json<Post>, ServerError> {
    let mut conn = pool.get().await.map_err(internal_server_error)?;

    let post = diesel::insert_into(posts::table)
        .values(post)
        .returning(Post::as_returning())
        .get_result(&mut conn)
        .await
        .map_err(internal_server_error)?;

    Ok(Json(post))
}

async fn list_posts(State(pool): State<AsyncPool>) -> Result<Json<Vec<Post>>, ServerError> {
    let mut conn = pool.get().await.map_err(internal_server_error)?;

    let posts = posts::table
        .filter(posts::dsl::published.eq(true))
        .load(&mut conn)
        .await
        .map_err(internal_server_error)?;

    Ok(Json(posts))
}

async fn get_post(State(pool): State<AsyncPool>, Path(post_id): Path<i32>) -> Result<Json<Post>, ServerError> {
    let mut conn = pool.get().await.map_err(internal_server_error)?;

    let post = posts::table
        .find(post_id)
        .first(&mut conn)
        .await
        .map_err(internal_server_error)?;

    Ok(Json(post))
}

async fn delete_post(State(pool): State<AsyncPool>, Path(post_id): Path<i32>) -> Result<(), ServerError> {
    let mut conn = pool.get().await.map_err(internal_server_error)?;

    diesel::delete(posts::table.find(post_id))
        .execute(&mut conn)
        .await
        .map_err(internal_server_error)?;

    Ok(())
}

fn internal_server_error<E: std::error::Error>(err: E) -> ServerError {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    // Set up the database connection
    // Format for DATABASE_URL=postgres://your_username:your_password@your_host:5432/your_db?sslmode=require
    let db_url = std::env::var("DATABASE_URL").expect("Env var `DATABASE_URL` not set");

    let mgr = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_setup(
        db_url,
        establish_connection,
    );

    let pool = Pool::builder()
        .max_size(10)
        .min_idle(Some(5))
        .max_lifetime(Some(Duration::from_secs(60 * 60 * 24)))
        .idle_timeout(Some(Duration::from_secs(60 * 2)))
        .build(mgr)
        .await?;
        
    // Set up the API routes
    let posts_api = Router::new()
        .route("/", get(list_posts).post(create_post))
        .route("/:id", get(get_post).delete(delete_post))
        .route("/get", get(list_posts))
        .route("/get/:id", get(get_post));
    let app = Router::new().nest("/posts", posts_api).with_state(pool);

    run(app).await
}


fn establish_connection(config: &str) -> BoxFuture<ConnectionResult<AsyncPgConnection>> {
    let fut = async {
        // We first set up the way we want rustls to work.
        let rustls_config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_certs())
            .with_no_client_auth();
        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(rustls_config);
        let (client, conn) = tokio_postgres::connect(config, tls)
            .await
            .map_err(|e| ConnectionError::BadConnection(e.to_string()))?;
        tokio::spawn(async move {
            if let Err(e) = conn.await {
                eprintln!("Database connection: {e}");
            }
        });
        AsyncPgConnection::try_from(client).await
    };
    fut.boxed()
}

fn root_certs() -> rustls::RootCertStore {
    let mut roots = rustls::RootCertStore::empty();
    let certs = rustls_native_certs::load_native_certs().expect("Certs not loadable!");
    let certs: Vec<_> = certs.into_iter().map(|cert| cert.0).collect();
    roots.add_parsable_certificates(&certs);
    roots
}
