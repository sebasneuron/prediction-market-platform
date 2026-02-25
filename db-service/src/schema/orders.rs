use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, PgPool, Postgres};
use utility_helpers::log_info;
use uuid::Uuid;

use crate::schema::enums::OrderType;

use super::enums::{OrderSide, OrderStatus, Outcome};

// need serialize for message pack
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub market_id: Uuid,
    pub side: OrderSide,
    pub outcome: Outcome,
    pub price: Decimal,
    pub quantity: Decimal,
    pub filled_quantity: Decimal,
    pub status: OrderStatus,
    pub order_type: OrderType,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// extend order struct with new fields
#[derive(Debug, Serialize, sqlx::FromRow, Clone)]
pub struct OrderWithMarket {
    pub id: Uuid,
    pub user_id: Uuid,
    pub market_id: Uuid,
    pub side: OrderSide,
    pub outcome: Outcome,
    pub price: Decimal,
    pub quantity: Decimal,
    pub filled_quantity: Decimal,
    pub status: OrderStatus,
    pub order_type: OrderType,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub liquidity_b: Decimal,
}

impl From<OrderWithMarket> for Order {
    fn from(order: OrderWithMarket) -> Self {
        Order {
            id: order.id,
            user_id: order.user_id,
            market_id: order.market_id,
            side: order.side,
            outcome: order.outcome,
            price: order.price,
            quantity: order.quantity,
            filled_quantity: order.filled_quantity,
            status: order.status,
            created_at: order.created_at,
            updated_at: order.updated_at,
            order_type: order.order_type,
        }
    }
}

impl Order {
    pub async fn create_order(
        user_id: Uuid,
        market_id: Uuid,
        price: Decimal,
        quantity: Decimal,
        side: OrderSide,
        outcome_side: Outcome,
        order_type: OrderType,
        pool: &PgPool,
    ) -> Result<Order, sqlx::Error> {
        let order = sqlx::query_as!(
            Order,
            r#"
            INSERT INTO "polymarket"."orders"
            (user_id, market_id, price, quantity, side, outcome, order_type)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING 
            id, user_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, filled_quantity,
            status as "status: OrderStatus",
            side as "side: OrderSide",            
            created_at, updated_at,
            order_type as "order_type: OrderType"
            "#,
            user_id,
            market_id,
            price,
            quantity,
            side as _,
            outcome_side as _,
            order_type as _,
        )
        .fetch_one(pool)
        .await?;

        log_info!("Order created - {:?}", order.id);
        Ok(order)
    }

    pub async fn delete_order_by_id(order_id: Uuid, pool: &PgPool) -> Result<Order, sqlx::Error> {
        let order = sqlx::query_as!(
            Order,
            r#"
            DELETE FROM polymarket.orders
            WHERE id = $1
            RETURNING 
            id, user_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, filled_quantity,
            status as "status: OrderStatus",
            side as "side: OrderSide",
            order_type as "order_type: OrderType",
            created_at, updated_at
            "#,
            order_id
        )
        .fetch_one(pool)
        .await?;

        log_info!("Order deleted - {:?}", order.id);
        Ok(order)
    }

    pub async fn update_order_status(
        order_id: Uuid,
        status: OrderStatus,
        pool: &PgPool,
    ) -> Result<Order, sqlx::Error> {
        let order = sqlx::query_as!(
            Order,
            r#"
            UPDATE polymarket.orders
            SET status = $1
            WHERE id = $2
            RETURNING 
            id, user_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, filled_quantity,
            status as "status: OrderStatus",
            side as "side: OrderSide",
            order_type as "order_type: OrderType",
            created_at, updated_at
            "#,
            status as _,
            order_id
        )
        .fetch_one(pool)
        .await?;

        log_info!("Order updated - {:?}", order.id);
        Ok(order)
    }

    pub async fn find_order_by_id(
        order_id: Uuid,
        pool: &PgPool,
    ) -> Result<Option<Order>, sqlx::Error> {
        let order = sqlx::query_as!(
            Order,
            r#"
            SELECT 
            id, user_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, filled_quantity,
            status as "status: OrderStatus",
            side as "side: OrderSide",
            order_type as "order_type: OrderType",
            created_at, updated_at    
            FROM polymarket.orders
            WHERE id = $1
            "#,
            order_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(order)
    }

    pub async fn find_order_by_id_and_status(
        order_id: Uuid,
        status: OrderStatus,
        pool: &PgPool,
    ) -> Result<Option<Order>, sqlx::Error> {
        let order = sqlx::query_as!(
            Order,
            r#"
            SELECT 
            id, user_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, filled_quantity,
            status as "status: OrderStatus",
            side as "side: OrderSide",
            order_type as "order_type: OrderType",
            created_at, updated_at            
            FROM polymarket.orders
            WHERE id = $1 AND status = $2
            "#,
            order_id,
            status as _
        )
        .fetch_optional(pool)
        .await?;

        Ok(order)
    }

    pub async fn find_order_by_id_with_market(
        order_id: Uuid,
        pool: &PgPool,
    ) -> Result<OrderWithMarket, sqlx::Error> {
        let order = sqlx::query_as!(
            OrderWithMarket,
            r#"
            SELECT 
            o.id, o.user_id, o.market_id,
            o.outcome as "outcome: Outcome",
            o.price, o.quantity, o.filled_quantity,
            o.status as "status: OrderStatus",
            o.side as "side: OrderSide",
            o.created_at, o.updated_at, m.liquidity_b,
            o.order_type as "order_type: OrderType"
            FROM polymarket.orders o
            LEFT JOIN polymarket.markets m ON o.market_id = m.id
            WHERE o.id = $1
            "#,
            order_id
        )
        .fetch_one(pool)
        .await?;

        log_info!("Order found - {:?}", order.id);
        Ok(order)
    }

    pub async fn get_all_open_orders(pool: &PgPool) -> Result<Vec<OrderWithMarket>, sqlx::Error> {
        let orders = sqlx::query_as!(
            OrderWithMarket,
            r#"
            SELECT 
            o.id, o.user_id, o.market_id,
            o.outcome as "outcome: Outcome",
            o.price, o.quantity, o.filled_quantity,
            o.status as "status: OrderStatus",
            o.side as "side: OrderSide",
            o.created_at, o.updated_at, m.liquidity_b,
            o.order_type as "order_type: OrderType"
            FROM polymarket.orders o
            JOIN polymarket.markets m ON o.market_id = m.id
            WHERE o.status = 'open'::polymarket.order_status         
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(orders)
    }

    pub async fn get_all_open_or_unspecified_orders(
        pool: &PgPool,
    ) -> Result<Vec<OrderWithMarket>, sqlx::Error> {
        let orders = sqlx::query_as!(
            OrderWithMarket,
            r#"
            SELECT 
            o.id, o.user_id, o.market_id,
            o.outcome as "outcome: Outcome",
            o.price, o.quantity, o.filled_quantity,
            o.status as "status: OrderStatus",
            o.side as "side: OrderSide",
            o.created_at, o.updated_at, m.liquidity_b,
            o.order_type as "order_type: OrderType"
            FROM polymarket.orders o
            JOIN polymarket.markets m ON o.market_id = m.id
            WHERE o.status IN ('open'::polymarket.order_status, 'unspecified'::polymarket.order_status)
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(orders)
    }

    pub async fn get_order_by_status(
        pool: &PgPool,
        status: OrderStatus,
    ) -> Result<Vec<OrderWithMarket>, sqlx::Error> {
        let orders = sqlx::query_as!(
            OrderWithMarket,
            r#"
            SELECT 
                o.id, o.user_id, o.market_id,
                o.outcome as "outcome: Outcome",
                o.price, o.quantity, o.filled_quantity,
                o.status as "status: OrderStatus",
                o.side as "side: OrderSide",
                o.created_at, o.updated_at, m.liquidity_b,
                o.order_type as "order_type: OrderType"
            FROM polymarket.orders o
            JOIN polymarket.markets m ON o.market_id = m.id                
            WHERE o.status = $1
            "#,
            status as _
        )
        .fetch_all(pool)
        .await?;

        Ok(orders)
    }

    pub async fn update(&self, pool: &PgPool) -> Result<Order, sqlx::Error> {
        let order = sqlx::query_as!(
            Order,
            r#"
            UPDATE "polymarket"."orders"
            SET 
                user_id = $1,
                market_id = $2,
                side = $3,
                outcome = $4,
                price = $5,
                quantity = $6,
                filled_quantity = $7,
                status = $8,
                order_type = $9
            WHERE id = $10
            RETURNING 
            id, user_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, filled_quantity,
            status as "status: OrderStatus",
            side as "side: OrderSide",
            order_type as "order_type: OrderType",
            created_at, updated_at
            "#,
            self.user_id,
            self.market_id,
            self.side as _,
            self.outcome as _,
            self.price,
            self.quantity,
            self.filled_quantity,
            self.status as _,
            self.order_type as _,
            self.id,
        )
        .fetch_one(pool)
        .await?;

        log_info!("Order updated - {:?}", order.id);
        Ok(order)
    }

    pub async fn get_buyer_and_seller_user_id(
        pg_pool: &sqlx::PgPool,
        buy_order_id: Uuid,
        sell_order_id: Uuid,
    ) -> Result<(Uuid, Uuid), sqlx::Error> {
        let order = sqlx::query!(
            r#"
            SELECT user_id FROM polymarket.orders
            WHERE id = $1 OR id = $2
            "#,
            buy_order_id,
            sell_order_id
        )
        .fetch_all(pg_pool)
        .await?;

        if order.len() != 2 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok((order[0].user_id, order[1].user_id))
    }

    pub async fn get_order_user_id(pool: &PgPool, order_id: Uuid) -> Result<Uuid, sqlx::Error> {
        let user_id = sqlx::query!(
            r#"
            SELECT user_id FROM polymarket.orders
            WHERE id = $1
            "#,
            order_id
        )
        .fetch_one(pool)
        .await?;

        Ok(user_id.user_id)
    }

    pub async fn update_order_status_and_filled_quantity(
        pool: &PgPool,
        order_id: Uuid,
        order_status: OrderStatus,
        new_filled_quantity: Decimal,
    ) -> Result<Order, sqlx::Error> {
        let order = sqlx::query_as!(
            Order,
            r#"
            UPDATE polymarket.orders
            SET status = $1, filled_quantity = $2
            WHERE id = $3
            RETURNING 
            id, user_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, filled_quantity,
            status as "status: OrderStatus",
            side as "side: OrderSide",
            order_type as "order_type: OrderType",
            created_at, updated_at
            "#,
            order_status as _,
            new_filled_quantity,
            order_id
        )
        .fetch_one(pool)
        .await?;

        log_info!("Order updated - {:?}", order.id);
        Ok(order)
    }

    pub async fn get_user_orders_by_paginated(
        pool: &PgPool,
        user_id: Uuid,
        status: OrderStatus,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<Order>, u32), sqlx::Error> {
        let offset = (page - 1) * page_size;

        let total_count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM polymarket.orders
            WHERE user_id = $1 AND status = $2
            "#,
            user_id,
            status as _
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        let total_pages = (total_count as u32 + page_size - 1) / page_size;

        let orders = sqlx::query_as!(
            Order,
            r#"
            SELECT
                id,
                user_id,
                market_id,
                outcome as "outcome: Outcome",
                price,
                quantity,
                filled_quantity,
                status as "status: OrderStatus",
                side as "side: OrderSide",
                order_type as "order_type: OrderType",
                created_at,
                updated_at
            FROM polymarket.orders
            WHERE user_id = $1 AND status = $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
            user_id,
            status as _,
            page_size as i64,
            offset as i64
        )
        .fetch_all(pool)
        .await?;

        Ok((orders, total_pages))
    }

    pub async fn get_user_orders_by_market_paginated(
        pool: &PgPool,
        user_id: Uuid,
        market_id: Uuid,
        page: u32,
        page_size: u32,
        status: Option<OrderStatus>,
    ) -> Result<(Vec<Order>, u32), sqlx::Error> {
        let offset = (page - 1) * page_size;
        let total_count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM polymarket.orders
            WHERE user_id = $1 AND market_id = $2
            "#,
            user_id,
            market_id
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        let total_pages = (total_count as u32 + page_size - 1) / page_size;

        if let Some(status) = status {
            let orders = sqlx::query_as!(
                Order,
                r#"
                    SELECT
                        id, user_id, market_id,
                        outcome as "outcome: Outcome",
                        price, 
                        quantity, 
                        filled_quantity,
                        status as "status: OrderStatus",
                        side as "side: OrderSide",
                        order_type as "order_type: OrderType",
                        created_at, updated_at
                    FROM polymarket.orders
                    WHERE user_id = $1 AND market_id = $2 AND status = $3
                    ORDER BY created_at DESC
                    LIMIT $4 OFFSET $5
                "#,
                user_id,
                market_id,
                status as _,
                page_size as i64,
                offset as i64
            )
            .fetch_all(pool)
            .await?;
            Ok((orders, total_pages))
        } else {
            let orders = sqlx::query_as!(
                Order,
                r#"
                    SELECT
                        id, user_id, market_id,
                        outcome as "outcome: Outcome",
                        price, 
                        quantity, 
                        filled_quantity,
                        status as "status: OrderStatus",
                        side as "side: OrderSide",
                        order_type as "order_type: OrderType",
                        created_at, updated_at
                    FROM polymarket.orders
                    WHERE user_id = $1 AND market_id = $2
                    ORDER BY created_at DESC
                    LIMIT $3 OFFSET $4
                "#,
                user_id,
                market_id,
                page_size as i64,
                offset as i64
            )
            .fetch_all(pool)
            .await?;
            Ok((orders, total_pages))
        }
    }

    pub async fn update_order_status_and_quantity(
        pool: &PgPool,
        order_id: Uuid,
        order_status: OrderStatus,
        new_quantity: Decimal,
    ) -> Result<Order, sqlx::Error> {
        let order = sqlx::query_as!(
            Order,
            r#"
            UPDATE polymarket.orders
            SET status = $1, quantity = $2
            WHERE id = $3
            RETURNING 
            id, user_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, filled_quantity,
            status as "status: OrderStatus",
            order_type as "order_type: OrderType",
            side as "side: OrderSide",
            created_at, updated_at
            "#,
            order_status as _,
            new_quantity,
            order_id
        )
        .fetch_one(pool)
        .await?;

        log_info!("Order updated - {:?}", order.id);
        Ok(order)
    }

    pub async fn insert_multiple_orders(
        orders: &Vec<Order>,
        pool: &PgPool,
    ) -> Result<Vec<Order>, sqlx::Error> {
        let mut transaction = pool.begin().await?;

        let mut inserted_orders = Vec::new();
        for order in orders {
            let inserted_order = sqlx::query_as!(
                Order,
                r#"
                INSERT INTO "polymarket"."orders"
                (user_id, market_id, price, quantity, side, outcome, order_type, status)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING 
                id, user_id, market_id,
                outcome as "outcome: Outcome",
                price, quantity, filled_quantity,
                status as "status: OrderStatus",
                side as "side: OrderSide",
                created_at, updated_at,
                order_type as "order_type: OrderType"        
                "#,
                order.user_id,
                order.market_id,
                order.price,
                order.quantity,
                order.side as _,
                order.outcome as _,
                order.order_type as _,
                order.status as _,
            )
            .fetch_one(&mut *transaction)
            .await?;

            inserted_orders.push(inserted_order);
        }

        transaction.commit().await?;
        Ok(inserted_orders)
    }

    pub async fn get_user_order_locked_funds(
        executor: impl Executor<'_, Database = Postgres>,
        user_id: Uuid,
    ) -> Result<Decimal, sqlx::Error> {
        let total_amount = sqlx::query_scalar!(
            r#"
            SELECT SUM((price * quantity) * 100) FROM polymarket.orders 
                WHERE user_id = $1 
                AND side = 'buy'::polymarket.order_side
                AND status = 'open'::polymarket.order_status
            "#,
            user_id,
        )
        .fetch_one(executor)
        .await?
        .unwrap_or(Decimal::ZERO);

        Ok(total_amount)
    }

    pub async fn get_user_locked_stokes(
        executor: impl Executor<'_, Database = Postgres>,
        user_id: Uuid,
        outcome_side: Outcome,
    ) -> Result<Decimal, sqlx::Error> {
        let locked_stokes = sqlx::query_scalar!(
            r#"
            SELECT SUM(quantity) FROM polymarket.orders 
                WHERE user_id = $1
                AND outcome = $2
                AND side = 'sell'::polymarket.order_side
                AND status = 'open'::polymarket.order_status
            "#,
            user_id,
            outcome_side as _
        )
        .fetch_one(executor)
        .await?
        .unwrap_or(Decimal::ZERO);
        Ok(locked_stokes)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::DateTime;
    use utility_helpers::types::GoogleClaims;

    use super::*;
    use crate::schema::{market::Market, users::User};

    #[tokio::test]
    // #[ignore = "just like this"]
    async fn test_create_order() {
        dotenv::dotenv().ok();
        let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
            .await
            .unwrap();

        let user = User::create_new_user(
            &pool,
            &GoogleClaims {
                email: "temp@gmail.com".to_string(),
                exp: 0,
                name: "temp".to_string(),
                picture: "temp".to_string(),
                sub: "temp".to_string(),
            },
        )
        .await
        .unwrap();

        let date_time = DateTime::parse_from_rfc3339("2025-06-20T12:28:33.675Z").unwrap();
        let market_expiry = date_time.naive_utc();

        let market = Market::create_new_market(
            "Test Market 0".to_string(),
            "Test Description".to_string(),
            "Test Logo".to_string(),
            Decimal::new(100, 2),
            market_expiry,
            &pool,
        )
        .await
        .unwrap();

        // values are taken from the database
        let user_id = user.id;
        let market_id = market.id;

        let price = Decimal::from_str("0.5").unwrap();
        let quantity = Decimal::from_str("1.0").unwrap();
        let side = OrderSide::BUY;

        let order = Order::create_order(
            user_id,
            market_id,
            price,
            quantity,
            side.clone(),
            Outcome::YES,
            OrderType::LIMIT,
            &pool,
        )
        .await
        .unwrap();

        assert_eq!(order.user_id, user_id);
        assert_eq!(order.market_id, market_id);
        assert_eq!(order.price, price);
        assert_eq!(order.quantity, quantity);
        assert_eq!(order.side, side);
        assert_eq!(order.filled_quantity, Decimal::ZERO);
        assert_eq!(order.status, OrderStatus::UNSPECIFIED);
        assert_eq!(order.outcome, Outcome::YES);
        assert_eq!(order.created_at, order.updated_at);

        // Clean up
        sqlx::query!(
            r#"
            DELETE FROM "polymarket"."orders"
            WHERE id = $1
            "#,
            order.id
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query!(
            r#"
            DELETE FROM "polymarket"."markets"
            WHERE id = $1
            "#,
            market.id
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query!(
            r#"
            DELETE FROM "polymarket"."users"
            WHERE id = $1
            "#,
            user.id
        )
        .execute(&pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_update_order_status_filled_quantity() {
        dotenv::dotenv().ok();
        let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
            .await
            .unwrap();

        let user = User::create_new_user(
            &pool,
            &GoogleClaims {
                email: "nami".to_string(),
                exp: 0,
                name: "nami".to_string(),
                picture: "nami".to_string(),
                sub: "nami".to_string(),
            },
        )
        .await
        .unwrap();

        let date_time = DateTime::parse_from_rfc3339("2025-06-20T12:28:33.675Z").unwrap();
        let market_expiry = date_time.naive_utc();

        let market = Market::create_new_market(
            "Test Market 0".to_string(),
            "Test Description".to_string(),
            "Test Logo".to_string(),
            Decimal::new(100, 2),
            market_expiry,
            &pool,
        )
        .await
        .unwrap();

        // values are taken from the database
        let user_id = user.id;
        let market_id = market.id;
        let price = Decimal::from_str("0.5").unwrap();
        let quantity = Decimal::from_str("1.0").unwrap();
        let side = OrderSide::BUY;
        let order = Order::create_order(
            user_id,
            market_id,
            price,
            quantity,
            side.clone(),
            Outcome::YES,
            OrderType::LIMIT,
            &pool,
        )
        .await
        .unwrap();

        assert_eq!(order.user_id, user_id);
        assert_eq!(order.market_id, market_id);
        assert_eq!(order.price, price);
        assert_eq!(order.quantity, quantity);
        assert_eq!(order.side, side);
        assert_eq!(order.filled_quantity, Decimal::ZERO);
        assert_eq!(order.status, OrderStatus::UNSPECIFIED);
        assert_eq!(order.outcome, Outcome::YES);

        // Update the order status to FILLED and set filled quantity
        let new_filled_quantity = Decimal::from_str("1.0").unwrap();
        let updated_order = Order::update_order_status_and_filled_quantity(
            &pool,
            order.id,
            OrderStatus::FILLED,
            new_filled_quantity,
        )
        .await
        .unwrap();

        assert_eq!(updated_order.id, order.id);
        assert_eq!(updated_order.status, OrderStatus::FILLED);
        assert_eq!(updated_order.filled_quantity, new_filled_quantity);

        // Clean up
        sqlx::query!(
            r#"
            DELETE FROM "polymarket"."orders"
            WHERE id = $1
            "#,
            updated_order.id
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query!(
            r#"
            DELETE FROM "polymarket"."markets"
            WHERE id = $1
            "#,
            market.id
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query!(
            r#"
            DELETE FROM "polymarket"."users"
            WHERE id = $1
            "#,
            user.id
        )
        .execute(&pool)
        .await
        .unwrap();

        log_info!("Order updated - {:?}", updated_order.id);
    }
}
