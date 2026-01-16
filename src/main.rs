mod contract;
mod vault_manager;
mod db;
mod balance_tracker;
mod api;

use dotenv::dotenv;
use std::sync::Arc;
use solana_sdk::signature::Keypair;
use warp::Filter;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    
    // Load configuration
    let rpc_url = std::env::var("RPC_URL")
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let payer_secret = std::env::var("PAYER_SECRET")
        .expect("PAYER_SECRET must be set");
    
    // Initialize keypair from secret
    let payer_keypair = Keypair::from_base58_string(&payer_secret);
    
    // Initialize contract client
    let contract = Arc::new(contract::VaultContract::new(&rpc_url, payer_keypair));
    
    // Initialize database
    let db = db::Database::new(&database_url).await;
    
    // Initialize vault manager
    let vault_manager = Arc::new(vault_manager::VaultManager::new(
        contract.clone(),
        db.pool.clone(),
    ));
    
    // Start balance tracker
    let balance_tracker = balance_tracker::BalanceTracker::new(
        db.clone(),
        contract.clone(),
        60, // Check every 60 seconds
    );
    tokio::spawn(async move {
        balance_tracker.start().await;
    });
    
    // Start API server
    let routes = api::routes(vault_manager);
    
    println!("Vault Backend Server running on http://localhost:3030");
    
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}