use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use db_service::schema::{
    enums::{OrderSide, OrderStatus, OrderType, Outcome},
    market::Market,
    orders::Order,
    user_holdings::UserHoldings,
    users::User,
};
use rand::Rng;
use rust_decimal::{Decimal, prelude::FromPrimitive};
use rust_decimal_macros::dec;
use serde::Deserialize;
use sqlx::types::chrono;
use utility_helpers::{
    log_error,
    message_pack_helper::serialize_to_message_pack,
    nats_helper::{NatsSubjects, types::InitializeOrderBookMessage},
};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct InitializeMarketPayload {
    market_id: Uuid,
    depth: u32,
    quantity: u32,
}

pub async fn initialize_market(
    State(state): State<AppState>,
    Json(payload): Json<InitializeMarketPayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let market_id = payload.market_id;
    let depth = payload.depth;
    let quantity = payload.quantity;

    let admin = User::get_or_create_admin(&state.pg_pool)
        .await
        .map_err(|e| {
            log_error!("Failed to get or create admin: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to get or create admin"})),
            )
        })?;

    let market = Market::get_market_by_id(&state.pg_pool, &market_id)
        .await
        .map_err(|e| {
            log_error!("Failed to get market by ID: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to get market"})),
            )
        })?;
    if market.is_none() {
        log_error!("Market with ID {} not found", market_id);
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Market not found"})),
        ));
    }
    let market = market.unwrap();

    // create user holdings
    let holding_future_one = UserHoldings::create_user_holding_conflict_free(
        &state.pg_pool,
        admin.id,
        market_id,
        dec!(1000),
        Outcome::YES,
    );
    let holding_future_two = UserHoldings::create_user_holding_conflict_free(
        &state.pg_pool,
        admin.id,
        market_id,
        dec!(1000),
        Outcome::NO,
    );

    let (yes_holdings, no_holdings) = tokio::try_join!(holding_future_one, holding_future_two)
        .map_err(|e| {
            log_error!("Failed to create user holdings: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create user holdings"})),
            )
        })?;

    /*
     * Each side of the market is initialized with `quantity` shares.
     * one price key can be used once only
     *
     * We should ensure that the orders must not matched with each other.
     */

    let random_orders = create_bootstrap_orders_with_stacked_price_levels(
        market_id,
        admin.id,
        depth,
        quantity,
        admin.balance,       // Admin balance
        yes_holdings.shares, // Yes holdings
        no_holdings.shares,  // No holdings
    );
    // insert orders into database
    Order::insert_multiple_orders(&random_orders, &state.pg_pool)
        .await
        .map_err(|e| {
            log_error!("Failed to insert orders: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to insert orders"})),
            )
        })?;

    let random_orders_count = random_orders.len();

    // preparing data for nats queue
    let order_book_initialize_data = InitializeOrderBookMessage {
        liquidity_b: market.liquidity_b,
        orders: random_orders,
    };
    let binary_payload = serialize_to_message_pack(&order_book_initialize_data).map_err(|e| {
        log_error!("Failed to serialize order book data: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to serialize order book data"})),
        )
    })?;

    let subject = NatsSubjects::InitializeOrderBook.to_string();

    state
        .jetstream
        .publish(subject, binary_payload.into())
        .await
        .map_err(|e| {
            log_error!("Failed to publish to NATS: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to publish to NATS"})),
            )
        })?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "Market initialized successfully",
            "market_id": market_id,
            "order_count": random_orders_count,
        })),
    ))
}

pub fn create_bootstrap_orders_with_stacked_price_levels(
    market_id: Uuid,
    admin_id: Uuid,
    depth: u32,
    quantity: u32,
    admin_balance: Decimal,
    yes_holdings: Decimal,
    no_holdings: Decimal,
) -> Vec<Order> {
    use rand::{Rng, seq::SliceRandom};
    use rust_decimal::Decimal;

    let mut rng = rand::rng();
    let now = chrono::Utc::now().naive_utc();

    let price_pairs: Vec<(Decimal, Decimal)> = (1..99)
        .step_by(2)
        .map(|p| (Decimal::new(p, 2), Decimal::new(p + 1, 2)))
        .collect();

    let mut shuffled = price_pairs.clone();
    shuffled.shuffle(&mut rng);

    let mut orders = vec![];
    let mut remaining_balance = admin_balance;
    let mut remaining_yes = yes_holdings;
    let mut remaining_no = no_holdings;

    for _ in 0..depth {
        let (buy_price, sell_price) = shuffled[rng.random_range(0..shuffled.len())];

        for _ in 0..quantity {
            // BUY YES
            let buy_yes_qty = random_qty();
            let buy_yes_cost = buy_price * buy_yes_qty;
            if remaining_balance >= buy_yes_cost {
                orders.push(Order {
                    id: Uuid::new_v4(),
                    user_id: admin_id,
                    market_id,
                    side: OrderSide::BUY,
                    outcome: Outcome::YES,
                    price: buy_price,
                    quantity: buy_yes_qty,
                    filled_quantity: Decimal::ZERO,
                    status: OrderStatus::OPEN,
                    order_type: OrderType::LIMIT,
                    created_at: now,
                    updated_at: now,
                });
                remaining_balance -= buy_yes_cost;
            }

            // SELL YES
            let sell_yes_qty = random_qty();
            if remaining_yes >= sell_yes_qty {
                orders.push(Order {
                    id: Uuid::new_v4(),
                    user_id: admin_id,
                    market_id,
                    side: OrderSide::SELL,
                    outcome: Outcome::YES,
                    price: sell_price,
                    quantity: sell_yes_qty,
                    filled_quantity: Decimal::ZERO,
                    status: OrderStatus::OPEN,
                    order_type: OrderType::LIMIT,
                    created_at: now,
                    updated_at: now,
                });
                remaining_yes -= sell_yes_qty;
            }

            // BUY NO
            let buy_no_qty = random_qty();
            let buy_no_cost = buy_price * buy_no_qty;
            if remaining_balance >= buy_no_cost {
                orders.push(Order {
                    id: Uuid::new_v4(),
                    user_id: admin_id,
                    market_id,
                    side: OrderSide::BUY,
                    outcome: Outcome::NO,
                    price: buy_price,
                    quantity: buy_no_qty,
                    filled_quantity: Decimal::ZERO,
                    status: OrderStatus::OPEN,
                    order_type: OrderType::LIMIT,
                    created_at: now,
                    updated_at: now,
                });
                remaining_balance -= buy_no_cost;
            }

            // SELL NO
            let sell_no_qty = random_qty();
            if remaining_no >= sell_no_qty {
                orders.push(Order {
                    id: Uuid::new_v4(),
                    user_id: admin_id,
                    market_id,
                    side: OrderSide::SELL,
                    outcome: Outcome::NO,
                    price: sell_price,
                    quantity: sell_no_qty,
                    filled_quantity: Decimal::ZERO,
                    status: OrderStatus::OPEN,
                    order_type: OrderType::LIMIT,
                    created_at: now,
                    updated_at: now,
                });
                remaining_no -= sell_no_qty;
            }
        }
    }

    orders
}

fn random_qty() -> Decimal {
    let mut rng = rand::rng();
    let q = rng.random_range(5.0..60.0);
    Decimal::from_f64(q).unwrap().round_dp(2)
}
