use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use anyhow::{Result, Context};
use std::time::Duration;

pub type DatabasePool = Pool<Postgres>;

pub async fn create_pool(database_url: &str) -> Result<DatabasePool> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(30))
        .connect(database_url)
        .await
        .context("Failed to create database pool")?;
    
    Ok(pool)
}