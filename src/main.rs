mod config;
mod auth;
mod routes;
mod schema;
mod shared;

use axum::Router;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

use config::Settings;

type DbPool = Pool<ConnectionManager<PgConnection>>;

fn setup_database(database_url: &str) -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder().build(manager).expect("FAILED to create DB pool")
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let settings = envy::from_env::<Settings>().expect("FAILED to read env vars");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let _pool = setup_database(&database_url);

    let app = routes::routes();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("FAILED to bind");

    tracing::info!("listening on 0.0.0.0:3000");

    axum::serve(listener, app)
        .await
        .expect("FAILED to serve");

    let _ = settings;
}
