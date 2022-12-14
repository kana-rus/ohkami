use ohkami::prelude::*;
use once_cell::sync::Lazy;
use serde::Serialize;
use sqlx::{FromRow, postgres::PgPoolOptions};

static DB_URL: Lazy<String> = Lazy::new(|| {
    format!("postgres://{}:{}@{}:{}/{}",
        std::env::var("POSTGRES_USER").unwrap(),
        std::env::var("POSTGRES_PASSWORD").unwrap(),
        std::env::var("POSTGRES_HOST").unwrap(),
        std::env::var("POSTGRES_PORT").unwrap(),
        std::env::var("POSTGRES_DB").unwrap(),
    )
});

fn main() -> Result<()> {
    let config = Config {
        db_profile: DBprofile {
            pool_options: PgPoolOptions::new().max_connections(20),
            url:          DB_URL.as_str(),
        },
        ..Config::default()
    };

    Server::setup_with(config)
        .GET("/api/users/:id", get_user_userid)
        .GET("/api/sleepy/users/:id", sleepy_get_user_userid)
        .serve_on(":3000")
}

#[derive(FromRow, Serialize)]
struct User {
    id:   i64,
    name: String,
}

async fn get_user_userid(ctx: Context, id: i64) -> Result<Response> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, name FROM users WHERE id = $1"
    ).bind(id)
        .fetch_one(ctx.pool())
        .await?;
    Response::OK(json(&user)?)
}

async fn sleepy_get_user_userid(ctx: Context, id: i64) -> Result<Response> {
    std::thread::sleep(std::time::Duration::from_secs(2));
    get_user_userid(ctx, id).await
}