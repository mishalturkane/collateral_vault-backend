use axum::{Router, Server};
use collateral_vault_backend::api;
use collateral_vault_backend::config::Config;
use collateral_vault_backend::database::DatabasePool;
use collateral_vault_backend::services;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "collateral_vault_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;
    
    // Initialize database pool
    let db_pool = DatabasePool::new(&config.database_url).await?;
    
    // Initialize services
    let rpc_service = services::rpc::RpcService::new(&config.rpc_url)?;
    let vault_service = services::vault::VaultService::new(
        db_pool.clone(),
        rpc_service,
        config.program_id,
        config.admin_keypair_path,
    )?;
    
    // Build application with routes
    let app = api::router::create_router(db_pool, vault_service, config.clone());
    
    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Server listening on {}", addr);
    
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}