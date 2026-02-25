use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::{Executor, Postgres};
use uuid::Uuid;

use crate::{
    pagination::PaginatedResponse,
    schema::enums::{MarketStatus, OrderSide},
};

use super::enums::Outcome;

#[derive(Debug, Serialize, sqlx::FromRow, Default)]
pub struct UserTrades {
    id: Uuid,
    // TODO: in free time change the field names `buy_order_id` and `sell_order_id` to `current_order_id` and `opposite_order_id`
    buy_order_id: Uuid,
    sell_order_id: Uuid,
    user_id: Uuid,
    market_id: Uuid,
    trade_type: OrderSide,
    outcome: Outcome,
    price: Decimal,
    quantity: Decimal,
    timestamp: NaiveDateTime,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserTradesWithMarket {
    // market
    pub market_name: String,
    pub market_logo: String,
    pub market_status: MarketStatus,
    pub market_final_outcome: Outcome,

    // trades
    pub trade_type: OrderSide,
    pub trade_outcome: Outcome,
    pub trade_price: Decimal,
    pub trade_quantity: Decimal,
}

#[derive(Debug, Serialize, sqlx::FromRow, Default)]
pub struct MarketTrades {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub avatar: String,
    pub trade_type: OrderSide,
    pub outcome: Outcome,
    pub price: Decimal,
    pub quantity: Decimal,
    pub timestamp: NaiveDateTime,
}

impl UserTrades {
    pub async fn create_user_trade<'a>(
        executor: impl Executor<'a, Database = Postgres>,
        current_order_id: Uuid,
        opposite_order_id: Uuid,
        user_id: Uuid,
        market_id: Uuid,
        outcome: Outcome,
        price: Decimal,
        quantity: Decimal,
        trade_type: OrderSide,
    ) -> Result<UserTrades, sqlx::error::Error> {
        let trade = sqlx::query_as!(
            UserTrades,
            r#"
            INSERT INTO polymarket.user_trades (buy_order_id, sell_order_id, user_id, market_id, outcome, price, quantity, trade_type)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, buy_order_id, sell_order_id, user_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, timestamp, created_at, updated_at,
            trade_type as "trade_type: OrderSide"
            "#,
            current_order_id,
            opposite_order_id,
            user_id,
            market_id,
            outcome as Outcome,
            price,
            quantity,
            trade_type as OrderSide,
        ).fetch_one(executor).await?;
        Ok(trade)
    }

    pub async fn get_market_trades_paginated(
        market_id: Uuid,
        admin_name: String,
        page: u64,
        page_size: u64,
        pool: &sqlx::PgPool,
    ) -> Result<PaginatedResponse<MarketTrades>, sqlx::Error> {
        let offset = (page - 1) * page_size;
        let total_count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM polymarket.user_trades ut
            JOIN polymarket.users u ON u.id = ut.user_id
            WHERE ut.market_id = $1 AND u.name != $2
            "#,
            market_id,
            admin_name
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        let trades = sqlx::query_as!(
            MarketTrades,
            r#"
            SELECT
                ut.id,
                u.name, 
                u.email, 
                u.avatar,
                ut.trade_type as "trade_type: OrderSide",
                ut.outcome as "outcome: Outcome", 
                ut.price, 
                ut.quantity, 
                ut.timestamp
            FROM polymarket.user_trades ut
            JOIN polymarket.users u ON u.id = ut.user_id
            WHERE ut.market_id = $1 AND u.name != $2
            ORDER BY ut.timestamp DESC
            LIMIT $3 OFFSET $4
            "#,
            market_id,
            admin_name,
            page_size as i64,
            offset as i64,
        )
        .fetch_all(pool)
        .await?;
        Ok(PaginatedResponse::new(
            trades,
            page,
            page_size,
            total_count as u64,
        ))
    }

    pub async fn get_user_trades_paginated(
        user_id: Uuid,
        page: u64,
        page_size: u64,
        pool: &sqlx::PgPool,
    ) -> Result<PaginatedResponse<UserTradesWithMarket>, sqlx::Error> {
        let offset = (page - 1) * page_size;
        let total_count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM polymarket.user_trades ut
            JOIN polymarket.markets m ON m.id = ut.market_id
            WHERE ut.user_id = $1
            "#,
            user_id
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        let trades = sqlx::query_as!(
            UserTradesWithMarket,
            r#"            
            SELECT 
                m.name AS market_name,
                m.logo AS market_logo,
                m.status AS "market_status: MarketStatus",
                m.final_outcome AS "market_final_outcome: Outcome",

                t.trade_type AS "trade_type: OrderSide",
                t.outcome AS "trade_outcome: Outcome",
                t.price AS trade_price,
                t.quantity AS trade_quantity
            FROM polymarket.user_trades t
            JOIN polymarket.markets m ON t.market_id = m.id
            WHERE t.user_id = $1
            ORDER BY t.timestamp DESC
            LIMIT $2 OFFSET $3;
            "#,
            user_id,
            page_size as i64,
            offset as i64,
        )
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(
            trades,
            page,
            page_size,
            total_count as u64,
        ))
    }
}
