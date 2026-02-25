use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utility_helpers::log_info;
use uuid::Uuid;

use super::enums::{MarketStatus, Outcome};
use crate::{
    pagination::PaginatedResponse,
    utils::{CronJobName, to_cron_expression},
};

// serialized by redis
#[derive(Debug, Serialize, sqlx::FromRow, Deserialize, Default)]
pub struct Market {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub logo: String,
    pub status: MarketStatus,
    pub liquidity_b: Decimal,
    pub final_outcome: Outcome,
    pub market_expiry: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Market {
    pub async fn create_new_market(
        name: String,
        description: String,
        logo: String,
        liquidity_b: Decimal,
        market_expiry: NaiveDateTime,
        pg_pool: &PgPool,
    ) -> Result<Self, sqlx::Error> {
        let mut tx = pg_pool.begin().await?;

        let market = sqlx::query_as!(
            Market,
            r#"
            INSERT INTO polymarket.markets (
                name,
                description,
                logo,
                liquidity_b,
                market_expiry
            ) VALUES (
                $1,
                $2,
                $3,
                $4,
                $5
            ) RETURNING 
                id,
                name,
                description,
                logo,
                status as "status: MarketStatus",
                final_outcome as "final_outcome: Outcome",
                liquidity_b,
                market_expiry,
                created_at,
                updated_at
            "#,
            name,
            description,
            logo,
            liquidity_b,
            market_expiry
        )
        .fetch_one(&mut *tx)
        .await?;

        // create cron
        let cron_name = CronJobName::CloseMarket(market.id).to_string();

        let cron_query = format!("SELECT polymarket.close_market('{}'::uuid);", market.id); // cron function
        let cron_expression = to_cron_expression(market.market_expiry);

        sqlx::query(
            r#"
            SELECT cron.schedule(
                $1, -- cron name
                $2, -- cron run time
                $3 -- cron function
            )
            "#,
        )
        .bind(cron_name)
        .bind(cron_expression)
        .bind(cron_query)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        log_info!("Market created: {}", market.id);
        Ok(market)
    }

    pub async fn get_all_markets(pg_pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let markets = sqlx::query_as!(
            Market,
            r#"
            SELECT 
                id,
                name,
                description,
                logo,
                status as "status: MarketStatus",
                final_outcome as "final_outcome: Outcome",
                liquidity_b,
                market_expiry,
                created_at,
                updated_at
            FROM "polymarket"."markets"
            "#,
        )
        .fetch_all(pg_pool)
        .await?;

        Ok(markets)
    }

    pub async fn get_all_markets_paginated(
        pg_pool: &PgPool,
        page: u64,
        page_size: u64,
    ) -> Result<PaginatedResponse<Self>, sqlx::Error> {
        let offset = (page - 1) * page_size;

        let total_count = sqlx::query!(
            r#"
            SELECT COUNT(*) as total_count
            FROM "polymarket"."markets"
            "#,
        )
        .fetch_one(pg_pool)
        .await?
        .total_count
        .unwrap_or(0);

        let markets = sqlx::query_as!(
            Market,
            r#"
            SELECT 
                id,
                name,
                description,
                logo,
                status as "status: MarketStatus",
                final_outcome as "final_outcome: Outcome",
                liquidity_b,
                market_expiry,
                created_at,
                updated_at
            FROM "polymarket"."markets"
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            page_size as i64,
            offset as i64
        )
        .fetch_all(pg_pool)
        .await?;

        Ok(PaginatedResponse::new(
            markets,
            page,
            page_size,
            total_count as u64,
        ))
    }

    pub async fn get_all_market_by_status_paginated(
        pg_pool: &PgPool,
        status: MarketStatus,
        page: u64,
        page_size: u64,
    ) -> Result<PaginatedResponse<Self>, sqlx::Error> {
        let offset = (page - 1) * page_size;

        let total_count = sqlx::query!(
            r#"
            SELECT COUNT(*) as total_count
            FROM "polymarket"."markets"
            WHERE status = $1
            "#,
            status as _
        )
        .fetch_one(pg_pool)
        .await?
        .total_count
        .unwrap_or(0);

        let markets = sqlx::query_as!(
            Market,
            r#"
            SELECT 
                id,
                name,
                description,
                logo,
                status as "status: MarketStatus",
                final_outcome as "final_outcome: Outcome",
                liquidity_b,
                market_expiry,
                created_at,
                updated_at
            FROM "polymarket"."markets"
            WHERE status = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            status as _,
            page_size as i64,
            offset as i64
        )
        .fetch_all(pg_pool)
        .await?;

        Ok(PaginatedResponse::new(
            markets,
            page,
            page_size,
            total_count as u64,
        ))
    }

    pub async fn get_market_by_id(
        pg_pool: &PgPool,
        market_id: &Uuid,
    ) -> Result<Option<Self>, sqlx::Error> {
        let market = sqlx::query_as!(
            Market,
            r#"
            SELECT 
                id,
                name,
                description,
                logo,
                status as "status: MarketStatus",
                final_outcome as "final_outcome: Outcome",
                liquidity_b,
                market_expiry,
                created_at,
                updated_at
            FROM "polymarket"."markets"
            WHERE id = $1
            "#,
            market_id
        )
        .fetch_optional(pg_pool)
        .await?;

        Ok(market)
    }

    pub async fn update_market_price(
        pg_pool: &PgPool,
        market_id: Uuid,
        price: Decimal,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE "polymarket"."markets" 
            SET liquidity_b = $1
            WHERE id = $2
            "#,
            price,
            market_id
        )
        .execute(pg_pool)
        .await?;

        Ok(())
    }

    pub async fn get_all_open_markets(pg_pool: &PgPool) -> Result<Vec<Market>, sqlx::Error> {
        let orders = sqlx::query_as!(
            Market,
            r#"
            SELECT 
                id,
                name,
                description,
                logo,
                status as "status: MarketStatus",
                final_outcome as "final_outcome: Outcome",
                liquidity_b,
                market_expiry,
                created_at,
                updated_at
            FROM polymarket.markets WHERE
            status = 'open'::polymarket.market_status;
            "#
        )
        .fetch_all(pg_pool)
        .await?;

        Ok(orders)
    }

    pub async fn settle_market(
        pg_pool: &PgPool,
        market_id: &Uuid,
        final_outcome: Outcome,
    ) -> Result<(), sqlx::Error> {
        let mut tx = pg_pool.begin().await?;

        // 1. Updating the market status to settled
        sqlx::query!(
            r#"
            UPDATE polymarket.markets
            SET status = 'settled'::polymarket.market_status,
                final_outcome = $2
            WHERE id = $1
            "#,
            market_id,
            final_outcome as _
        )
        .execute(&mut *tx)
        .await?;

        // 2. Expiring all open orders in the market
        sqlx::query!(
            r#"
            UPDATE polymarket.orders
            SET status = 'expired'::polymarket.order_status
            WHERE market_id = $1 AND status in (
                'open'::polymarket.order_status,
                'partial_fill'::polymarket.order_status,
                'pending_update'::polymarket.order_status,
                'pending_cancel'::polymarket.order_status
            )
            "#,
            market_id
        )
        .execute(&mut *tx)
        .await?;

        // 3. Credit the balance to the user's holdings
        sqlx::query!(
            r#"
            UPDATE polymarket.users u
            SET balance = balance + (payout.total * 100) -- Each share is worth 100 after settlement
            FROM (
                SELECT user_id, SUM(shares) AS total
                FROM polymarket.user_holdings
                WHERE market_id = $1 AND outcome = $2
                GROUP BY user_id
            ) AS payout
             WHERE u.id = payout.user_id
            "#,
            market_id,
            final_outcome as _
        )
        .execute(&mut *tx)
        .await?;

        // 4. Zero out all the holdings for the market
        sqlx::query!(
            r#"
            UPDATE polymarket.user_holdings
            SET shares = 0
            WHERE market_id = $1
            "#,
            market_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use chrono::DateTime;

    use super::*;

    #[tokio::test]
    async fn test_create_new_market() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pg_pool = PgPool::connect(&database_url).await.unwrap();

        let date_time = DateTime::parse_from_rfc3339("2025-06-20T12:28:33.675Z").unwrap();
        let market_expiry = date_time.naive_utc();

        let market = Market::create_new_market(
            "Test Market 0".to_string(),
            "Test Description".to_string(),
            "Test Logo".to_string(),
            Decimal::new(100, 2),
            market_expiry,
            &pg_pool,
        )
        .await
        .unwrap();

        assert_eq!(market.name, "Test Market 0");
        assert_eq!(market.description, "Test Description");
        assert_eq!(market.logo, "Test Logo");
        assert_eq!(market.liquidity_b, Decimal::new(100, 2));
        // Clean up the test market
        sqlx::query!(
            r#"
            DELETE FROM "polymarket"."markets" 
            WHERE id = $1
            "#,
            market.id
        )
        .execute(&pg_pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_get_all_markets() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pg_pool = PgPool::connect(&database_url).await.unwrap();

        let markets = Market::get_all_markets(&pg_pool).await;

        assert!(markets.is_ok());
    }

    #[tokio::test]
    async fn test_get_all_markets_paginated() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pg_pool = PgPool::connect(&database_url).await.unwrap();

        let paginated_response = Market::get_all_markets_paginated(&pg_pool, 1, 10)
            .await
            .unwrap();
        assert_eq!(paginated_response.page_info.page, 1);
        assert_eq!(paginated_response.page_info.page_size, 10);
    }

    #[tokio::test]
    async fn test_get_market_by_id() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pg_pool = PgPool::connect(&database_url).await.unwrap();

        let date_time = DateTime::parse_from_rfc3339("2025-06-20T12:28:33.675Z").unwrap();
        let market_expiry = date_time.naive_utc();

        let market = Market::create_new_market(
            "Test Market 0".to_string(),
            "Test Description".to_string(),
            "Test Logo".to_string(),
            Decimal::new(100, 2),
            market_expiry,
            &pg_pool,
        )
        .await
        .unwrap();

        let fetched_market = Market::get_market_by_id(&pg_pool, &market.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(fetched_market.id, market.id);
        assert_eq!(fetched_market.name, market.name);
        assert_eq!(fetched_market.description, market.description);
        assert_eq!(fetched_market.logo, market.logo);
        assert_eq!(fetched_market.liquidity_b, market.liquidity_b);

        // Clean up the test market
        sqlx::query!(
            r#"
            DELETE FROM "polymarket"."markets" 
            WHERE id = $1
            "#,
            market.id
        )
        .execute(&pg_pool)
        .await
        .unwrap();
    }
}
