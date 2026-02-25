use async_nats::jetstream;
use auth_service::types::SessionTokenClaims;
use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use db_service::schema::{
    enums::{MarketStatus, OrderSide, OrderStatus, OrderType, Outcome},
    market::Market,
    orders::Order,
    user_holdings::UserHoldings,
    users::User,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::json;
use utility_helpers::{
    log_error, log_info,
    message_pack_helper::serialize_to_message_pack,
    nats_helper::{NatsSubjects, types::MarketOrderCreateMessage},
};
use uuid::Uuid;

use crate::{require_field, state::AppState};

#[derive(Debug, Deserialize)]
pub struct MarketOrderPayload {
    pub market_id: Option<Uuid>,
    pub price: Option<Decimal>,
    pub outcome: Option<Outcome>,
    pub side: Option<OrderSide>,
}

pub async fn create_limit_order(
    State(app_state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
    Json(payload): Json<MarketOrderPayload>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let market_id = payload.market_id;
    let budget = payload.price;
    let outcome = payload.outcome;
    let side = payload.side;

    require_field!(market_id);
    require_field!(budget);
    require_field!(outcome);
    require_field!(side);

    let market_id = market_id.unwrap();

    let market = Market::get_market_by_id(&app_state.pg_pool, &market_id)
        .await
        .map_err(|e| {
            log_error!("Failed to get market - {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to get market"
                }))
                .into_response(),
            )
        })?;

    if market.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Market not found"
            }))
            .into_response(),
        ));
    }

    if market.unwrap().status == MarketStatus::SETTLED {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Market is already settled, cannot create order"
            }))
            .into_response(),
        ));
    }

    let budget = budget.unwrap();
    let outcome = outcome.unwrap();
    let side = side.unwrap();
    let user_id = claims.user_id;

    app_state
        .jetstream
        .get_or_create_stream(jetstream::stream::Config {
            name: "ORDER".into(),
            subjects: vec!["order.>".into()],
            ..Default::default()
        })
        .await
        .map_err(|e| {
            log_error!("Failed to get or create stream: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to initialize message stream"})).into_response(),
            )
        })?;

    if side == OrderSide::SELL {
        let holdings = UserHoldings::get_user_holdings_by_outcome(
            &app_state.pg_pool,
            user_id,
            market_id,
            outcome,
        )
        .await
        .map_err(|e| {
            log_error!("Failed to get user holdings: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to retrieve user holdings"})).into_response(),
            )
        })?;

        if holdings.shares <= Decimal::ZERO {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Insufficient shares to place a buy order"})).into_response(),
            ));
        }
    } else {
        let mut tx = app_state.pg_pool.begin().await.map_err(|e| {
            log_error!("Failed to begin transaction - {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to begin transaction"
                }))
                .into_response(),
            )
        })?;
        let balance = User::get_user_balance(&mut *tx, user_id)
            .await
            .map_err(|e| {
                log_error!("Failed to get user balance: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to retrieve user balance"})).into_response(),
                )
            })?;

        let total_user_locked_funds = Order::get_user_order_locked_funds(&mut *tx, claims.user_id)
            .await
            .map_err(|e| {
                log_error!("Failed to get user orders total amount - {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Failed to get user orders total amount"
                    }))
                    .into_response(),
                )
            })?;

        tx.commit().await.map_err(|e| {
            log_error!("Failed to commit transaction - {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to commit transaction"
                }))
                .into_response(),
            )
        })?;

        if total_user_locked_funds + budget > balance {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Insufficient balance to place a buy order"})).into_response(),
            ));
        }

        if balance < budget {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Insufficient balance to place a sell order"}))
                    .into_response(),
            ));
        }
    }

    let order = Order::create_order(
        user_id,
        market_id,
        Decimal::ZERO,
        Decimal::ZERO,
        side,
        outcome,
        OrderType::MARKET,
        &app_state.pg_pool,
    )
    .await
    .map_err(|e| {
        log_error!("Failed to create order: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to create order"})).into_response(),
        )
    })?;

    let market_order_create_message = MarketOrderCreateMessage {
        order_id: order.id,
        budget: budget / Decimal::new(100, 0),
    };

    let message_pack_encoded_message = serialize_to_message_pack(&market_order_create_message)
        .map_err(|e| {
            log_error!("Failed to serialize market order create message: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to serialize market order create message"}))
                    .into_response(),
            )
        })?;

    let subject = NatsSubjects::MarketOrderCreate;
    let mut is_failed = false;

    app_state
        .jetstream
        .publish(subject.to_string(), message_pack_encoded_message.into())
        .await
        .map_err(|e| {
            log_error!("Failed to publish market order create message: {}", e);
            is_failed = true;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to publish market order create message"}))
                    .into_response(),
            )
        })?;

    if is_failed {
        Order::update_order_status(order.id, OrderStatus::CANCELLED, &app_state.pg_pool)
            .await
            .map_err(|e| {
                log_error!("Failed to update order status to cancelled: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to update order status to cancelled"}))
                        .into_response(),
                )
            })?;
    }

    log_info!(
        "Market order created and published to jetstream - {:?}",
        order.id
    );

    let response = json!({
        "message": "Market order created successfully",
        "order": {
            "id": order.id,
            "user_id": order.user_id,
            "market_id": order.market_id,
            "budget": budget,
            "outcome": outcome,
            "side": side,
            "status": order.status,
        },
    });

    Ok((StatusCode::CREATED, Json(response)))
}
