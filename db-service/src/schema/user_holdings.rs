use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::{Executor, Postgres};
use uuid::Uuid;

use crate::{
    pagination::PaginatedResponse,
    schema::enums::{MarketStatus, Outcome},
};

#[derive(Debug, Serialize, sqlx::FromRow, Default)]
pub struct UserHoldings {
    pub id: Uuid,
    pub user_id: Uuid,
    pub market_id: Uuid,
    pub shares: Decimal,
    pub outcome: Outcome,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserIdWithShares {
    pub user_id: Uuid,
    pub total_shares: Option<Decimal>,
    pub total_yes_shares: Option<Decimal>,
    pub total_no_shares: Option<Decimal>,
    pub username: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct UserHoldingWithMarket {
    pub market_id: Uuid,
    pub outcome: Outcome,
    pub shares: Decimal,

    pub market_name: String,
    pub market_description: String,
    pub market_logo: String,
    pub market_status: MarketStatus,
    pub final_outcome: Outcome,
    pub market_expiry: NaiveDateTime,
    pub market_created_at: NaiveDateTime,
    pub market_updated_at: NaiveDateTime,
}

impl UserHoldings {
    pub async fn create_user_holding<'a>(
        executor: impl Executor<'a, Database = Postgres>,
        user_id: Uuid,
        market_id: Uuid,
        shares: Decimal,
        outcome: Outcome,
    ) -> Result<UserHoldings, sqlx::error::Error> {
        let holding = sqlx::query_as!(
            UserHoldings,
            r#"
            INSERT INTO polymarket.user_holdings (user_id, market_id, shares, outcome)
            VALUES ($1, $2, $3, $4)
            RETURNING 
                id, 
                user_id, 
                market_id, 
                shares, 
                created_at, 
                updated_at, 
                outcome as "outcome: Outcome";
            "#,
            user_id,
            market_id,
            shares,
            outcome as _
        )
        .fetch_one(executor)
        .await?;

        Ok(holding)
    }

    pub async fn create_user_holding_conflict_free<'a>(
        executor: impl Executor<'a, Database = Postgres>,
        user_id: Uuid,
        market_id: Uuid,
        shares: Decimal,
        outcome: Outcome,
    ) -> Result<UserHoldings, sqlx::error::Error> {
        let holding = sqlx::query_as!(
            UserHoldings,
            r#"
            INSERT INTO polymarket.user_holdings (user_id, market_id, shares, outcome)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id, market_id, outcome)
            DO UPDATE SET shares = polymarket.user_holdings.shares + $3,
            updated_at = NOW()            
            RETURNING 
                id, 
                user_id, 
                market_id, 
                shares, 
                created_at, 
                updated_at, 
                outcome as "outcome: Outcome";
            "#,
            user_id,
            market_id,
            shares,
            outcome as _
        )
        .fetch_one(executor)
        .await?;

        Ok(holding)
    }

    pub async fn update_user_holdings<'a>(
        executor: impl Executor<'a, Database = Postgres>,
        user_id: Uuid,
        market_id: Uuid,
        quantity: Decimal,
        outcome: Outcome,
    ) -> Result<UserHoldings, sqlx::error::Error> {
        let holding = sqlx::query_as!(
            UserHoldings,
            r#"
            INSERT INTO polymarket.user_holdings (user_id, market_id, shares, outcome)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id, market_id, outcome)
            DO UPDATE SET shares = polymarket.user_holdings.shares + $3,
            updated_at = NOW()
            RETURNING 
                id, 
                user_id, 
                market_id, 
                shares, 
                created_at, 
                updated_at, 
                outcome as "outcome: Outcome";
            "#,
            user_id,
            market_id,
            quantity,
            outcome as _
        )
        .fetch_one(executor)
        .await?;

        Ok(holding)
    }

    pub async fn get_user_holdings_by_outcome(
        db_pool: &sqlx::PgPool,
        user_id: Uuid,
        market_id: Uuid,
        outcome: Outcome,
    ) -> Result<UserHoldings, sqlx::error::Error> {
        let mut tx = db_pool.begin().await?;
        println!("Outcome: {:?}", outcome);

        // making sure the user holding exists, as on initial new order creation, it might not exist
        sqlx::query!(
            r#"
            INSERT INTO polymarket.user_holdings (user_id, market_id, shares, outcome)
            VALUES ($1, $2, 0, $3)
            ON CONFLICT (user_id, market_id, outcome) DO NOTHING
            "#,
            user_id,
            market_id,
            outcome as _
        )
        .execute(&mut *tx)
        .await?;

        let holdings = sqlx::query_as!(
            UserHoldings,
            r#"
            SELECT id, user_id, market_id, shares, created_at, updated_at, outcome as "outcome: Outcome"
            FROM polymarket.user_holdings
            WHERE user_id = $1 AND market_id = $2 AND outcome = $3
            "#,
            user_id,
            market_id,
            outcome as _
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(holdings)
    }

    pub async fn get_user_holdings(
        db_pool: &sqlx::PgPool,
        user_id: Uuid,
        market_id: Uuid,
    ) -> Result<Vec<UserHoldings>, sqlx::error::Error> {
        sqlx::query_as!(
            UserHoldings,
            r#"
            SELECT id, user_id, market_id, shares, created_at, updated_at, outcome as "outcome: Outcome"
            FROM polymarket.user_holdings
            WHERE user_id = $1 AND market_id = $2
            "#,
            user_id,
            market_id
        )
        .fetch_all(db_pool)
        .await
    }

    pub async fn get_user_holdings_sum_both_outcome(
        db_pool: &sqlx::PgPool,
        user_id: Uuid,
        market_id: Uuid,
    ) -> Result<(Decimal, Decimal), sqlx::error::Error> {
        let result = sqlx::query!(
            r#"
            SELECT 
                SUM(CASE WHEN outcome = 'yes'::polymarket.outcome THEN shares ELSE 0 END) as "yes_shares",
                SUM(CASE WHEN outcome = 'no'::polymarket.outcome THEN shares ELSE 0 END) as "no_shares"
            FROM polymarket.user_holdings
            WHERE user_id = $1 AND market_id = $2
            "#,
            user_id,
            market_id
        )
        .fetch_one(db_pool)
        .await?;

        let yes_shares = result.yes_shares.unwrap_or(Decimal::ZERO);
        let no_shares = result.no_shares.unwrap_or(Decimal::ZERO);
        Ok((yes_shares, no_shares))
    }

    pub async fn get_top_holders(
        db_pool: &sqlx::PgPool,
        market_id: Uuid,
        admin_user: String,
        limit: i8,
    ) -> Result<Vec<UserIdWithShares>, sqlx::error::Error> {
        let orders = sqlx::query_as!(
            UserIdWithShares,
            r#"
                SELECT
                    u.id AS user_id,
                    u.name AS username,
                    u.avatar,
                    SUM(uh.shares) AS total_shares,
                    SUM(uh.shares) FILTER (WHERE uh.outcome = 'yes'::polymarket.outcome) AS total_yes_shares,
                    SUM(uh.shares) FILTER (WHERE uh.outcome = 'no'::polymarket.outcome) AS total_no_shares
                FROM polymarket.user_holdings uh
                JOIN polymarket.users u ON uh.user_id = u.id
                WHERE uh.market_id = $1 AND u.name != $2
                GROUP BY u.id, u.name, u.avatar
                ORDER BY total_shares DESC
                LIMIT $3
            "#,
            market_id,
            admin_user,
            limit as i32
        )
        .fetch_all(db_pool)
        .await?;

        Ok(orders)
    }

    pub async fn get_user_holdings_by_market_paginated(
        user_id: Uuid,
        page: u64,
        page_size: u64,
        db_pool: &sqlx::PgPool,
    ) -> Result<PaginatedResponse<UserHoldingWithMarket>, sqlx::error::Error> {
        let offset = (page - 1) * page_size;
        let total_count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM polymarket.user_holdings uh
            JOIN polymarket.markets m ON uh.market_id = m.id
            WHERE uh.user_id = $1
            "#,
            user_id
        )
        .fetch_one(db_pool)
        .await?
        .unwrap_or(0);

        let user_holdings = sqlx::query_as!(
            UserHoldingWithMarket,
            r#"
            SELECT 
                uh.market_id,
                uh.outcome AS "outcome: Outcome",
                uh.shares,
                
                m.name AS market_name,
                m.description AS market_description,
                m.logo AS market_logo,
                m.status AS "market_status: MarketStatus",
                m.final_outcome AS "final_outcome: Outcome",
                m.market_expiry AS market_expiry,
                m.created_at AS market_created_at,
                m.updated_at AS market_updated_at
            FROM polymarket.user_holdings uh
            JOIN polymarket.markets m ON uh.market_id = m.id
            WHERE uh.user_id = $1
            ORDER BY uh.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            page_size as i64,
            offset as i64,
        )
        .fetch_all(db_pool)
        .await?;

        Ok(PaginatedResponse::new(
            user_holdings,
            page,
            page_size,
            total_count as u64,
        ))
    }
}
