use std::error::Error as StdError;

use async_nats::{
    connect,
    jetstream::{self, Context},
};
use auth_service::AuthService;
use db_service::DbService;
use utility_helpers::{log_info, redis::RedisHelper, types::EnvVarConfig};

use crate::bloom_f::BloomFilterWrapper;

#[derive(Clone)]
pub struct AppState {
    pub pg_pool: sqlx::PgPool,
    pub auth_service: AuthService,
    pub jetstream: Context,
    pub bloom_filter: BloomFilterWrapper, // already thread safe
    pub redis_helper: RedisHelper,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn StdError>> {
        dotenv::dotenv().ok();

        let env_var_config = EnvVarConfig::new()?;

        let ns = connect(&env_var_config.nc_url).await?;
        let jetstream = jetstream::new(ns);

        let pg_pool = sqlx::PgPool::connect(&env_var_config.database_url).await?;
        let auth_service = AuthService::new(pg_pool.clone())?;

        let bloom_filter = BloomFilterWrapper::new(&pg_pool).await?;
        let redis_helper = RedisHelper::new(
            &env_var_config.redis_url,
            60 * 60, // default cache expiration 60 sec * 60 sec = 1 hour
        )
        .await?;

        let state = AppState {
            pg_pool,
            auth_service,
            jetstream,
            bloom_filter,
            redis_helper,
        };

        Ok(state)
    }

    pub async fn run_migrations(&self) -> Result<(), Box<dyn StdError>> {
        DbService::run_migrations(&self.pg_pool)
            .await
            .map_err(|e| format!("Migration failed: {}", e))?;

        log_info!("Database migrations completed successfully.");

        Ok(())
    }
}
