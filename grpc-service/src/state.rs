use std::sync::Arc;

use sqlx::PgPool;
use utility_helpers::{log_info, redis::RedisHelper, types::EnvVarConfig};

pub type SafeState = Arc<AppState>;
pub struct AppState {
    pub db_pool: PgPool,
    pub redis_helper: RedisHelper,
    pub clickhouse_client: clickhouse::Client,
    pub admin_username: String,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let env_config = EnvVarConfig::new()?;
        let redis_helper = RedisHelper::new(&env_config.redis_url, 60).await?; // 60 seconds TTL for Redis keys
        log_info!("Connected to Redis");

        let db_pool = PgPool::connect(&env_config.database_url).await?;
        log_info!("Connected to Postgres");

        let clickhouse_client = clickhouse::Client::default()
            .with_url(env_config.clickhouse_url)
            .with_database("polyMarket")
            .with_user("polyMarket")
            .with_password(env_config.clickhouse_password);
        let admin_username = env_config.admin_username.clone();
        log_info!("Connected to ClickHouse");

        Ok(AppState {
            admin_username,
            db_pool,
            redis_helper,
            clickhouse_client,
        })
    }
}
