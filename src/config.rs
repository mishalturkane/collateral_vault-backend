use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub rpc_url: String,
    pub ws_url: String,
    pub program_id: String,
    pub admin_keypair_path: PathBuf,
    pub jwt_secret: String,
    pub cors_origins: Vec<String>,
    pub rate_limit_requests: u64,
    pub rate_limit_duration: u64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenv::dotenv().ok();
        
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()?;
        
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        
        let redis_url = env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        
        let rpc_url = env::var("RPC_URL")
            .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
        
        let ws_url = env::var("WS_URL")
            .unwrap_or_else(|_| "wss://api.devnet.solana.com".to_string());
        
        let program_id = env::var("PROGRAM_ID")
            .unwrap_or_else(|_| "G6TF8EdpP7gKwfPmNEhMLU7E34X5Fr3ujpAMdCzwHz8R".to_string());
        
        let admin_keypair_path = PathBuf::from(
            env::var("ADMIN_KEYPAIR_PATH")
                .unwrap_or_else(|_| "./admin-keypair.json".to_string())
        );
        
        let jwt_secret = env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set");
        
        let cors_origins = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "*".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        let rate_limit_requests = env::var("RATE_LIMIT_REQUESTS")
            .unwrap_or_else(|_| "100".to_string())
            .parse()?;
        
        let rate_limit_duration = env::var("RATE_LIMIT_DURATION")
            .unwrap_or_else(|_| "3600".to_string())
            .parse()?;
        
        Ok(Self {
            port,
            database_url,
            redis_url,
            rpc_url,
            ws_url,
            program_id,
            admin_keypair_path,
            jwt_secret,
            cors_origins,
            rate_limit_requests,
            rate_limit_duration,
        })
    }
}