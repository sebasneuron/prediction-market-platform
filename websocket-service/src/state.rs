use async_nats::{connect, jetstream};
use tokio::sync::RwLock;
use utility_helpers::{log_info, types::EnvVarConfig};

use crate::core::client_manager::SubscriptionAndClientManager;

#[derive(Debug)]
pub struct WebSocketAppState {
    pub client_manager: RwLock<SubscriptionAndClientManager>,
    pub jetstream: jetstream::Context,
}

impl WebSocketAppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();
        let env_var_config = EnvVarConfig::new()?;

        let nc = connect(&env_var_config.nc_url)
            .await
            .expect("Failed to connect to NATS server");
        log_info!("Connected to NATS");
        let jetstream = jetstream::new(nc);

        Ok(WebSocketAppState {
            jetstream,
            client_manager: RwLock::new(SubscriptionAndClientManager::new()),
        })
    }
}
