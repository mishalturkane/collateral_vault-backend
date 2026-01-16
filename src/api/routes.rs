use axum::{
    Router,
    routing::{get, post, delete},
    middleware,
};
use axum::extract::State;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::database::DatabasePool;
use crate::services::vault::VaultService;

pub fn create_router(
    db_pool: DatabasePool,
    vault_service: VaultService,
    config: Config,
) -> Router {
    let api_v1_router = Router::new()
        // Health check
        .route("/health", get(handlers::health_check))
        
        // Vault operations
        .route("/vaults", post(handlers::create_vault))
        .route("/vaults/:owner", get(handlers::get_vault))
        .route("/vaults/:owner/deposit", post(handlers::deposit))
        .route("/vaults/:owner/withdraw", post(handlers::withdraw))
        .route("/vaults/:owner/close", post(handlers::close_vault))
        
        // Collateral operations
        .route("/vaults/:owner/lock", post(handlers::lock_collateral))
        .route("/vaults/:owner/unlock", post(handlers::unlock_collateral))
        .route("/vaults/:owner/transfer", post(handlers::transfer_collateral))
        
        // Admin operations
        .route("/admin/authority", post(handlers::initialize_authority))
        .route("/admin/authority/programs", post(handlers::add_authorized_program))
        .route("/admin/authority/programs/:program", delete(handlers::remove_authorized_program))
        
        // Transaction endpoints
        .route("/transactions/build/:tx_type", post(handlers::build_transaction))
        .route("/transactions/submit", post(handlers::submit_transaction))
        .route("/transactions/:signature", get(handlers::get_transaction_status))
        
        // Event stream
        .route("/events/stream", get(handlers::stream_events))
        
        .layer(middleware::from_fn(handlers::auth_middleware))
        .with_state((db_pool, vault_service));

    Router::new()
        .nest("/api/v1", api_v1_router)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}