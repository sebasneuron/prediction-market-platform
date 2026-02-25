use std::error::Error as StdError;

pub mod pagination;
pub mod procedures;
pub mod schema;
pub mod utils;

pub struct DbService;

impl DbService {
    pub async fn run_migrations(pg_pool: &sqlx::PgPool) -> Result<(), Box<dyn StdError>> {
        sqlx::migrate!("./migrations")
            .run(pg_pool)
            .await
            .map_err(|e| format!("Migration failed: {}", e))?;

        tracing::info!("Database migrations completed successfully.");
        Ok(())
    }
}
