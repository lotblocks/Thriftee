use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub stripe_secret_key: String,
    pub stripe_webhook_secret: String,
    pub blockchain_rpc_url: String,
    pub blockchain_ws_url: String,
    pub contract_address: String,
    pub deployer_private_key: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        dotenvy::dotenv().ok();

        let config = config::Config::builder()
            .set_default("host", "127.0.0.1")?
            .set_default("port", 8080)?
            .add_source(config::Environment::default())
            .build()?;

        config.try_deserialize()
    }
}