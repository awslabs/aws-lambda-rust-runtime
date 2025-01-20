use axum::{
    extract::{Path, State},
    response::Json,
    routing::get,
    Router,
};
use diesel::prelude::*;
use diesel_async::{
    pooled_connection::{bb8::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection, RunQueryDsl,
};
use lambda_http::{http::StatusCode, run, tracing, Error};
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

type AsyncPool = Pool<AsyncPgConnection>;
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
    tracing::init_default_subscriber();

    // Set up the database connection
    let db_url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL environment variable");
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(db_url);
    let connection = Pool::builder()
        .build(config)
        .await
        .expect("unable to establish the database connection");

    // Set up the API routes
    let posts_api = Router::new()
        .route("/", get(list_posts).post(create_post))
        .route("/:id", get(get_post).delete(delete_post));
    let app = Router::new().nest("/posts", posts_api).with_state(connection);

    run(app).await
}
