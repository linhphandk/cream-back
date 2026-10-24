use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use tracing::info;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn establish_pool(database_url: &str, pool_size: u32) -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .max_size(pool_size)
        .build(manager)
        .expect("Failed to create database pool");
    info!(pool_size, "database pool ready");
    pool
}
