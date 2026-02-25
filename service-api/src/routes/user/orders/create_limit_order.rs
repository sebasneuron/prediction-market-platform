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
use rust_decimal::{Decimal, prelude::FromPrimitive};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::types::Uuid;
use utility_helpers::{log_error, log_info, nats_helper::NatsSubjects};

use crate::{require_field, state::AppState};

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateOrderPayload {
    market_id: Option<Uuid>,
    price: Option<u8>,
    quantity: Option<f64>,
    side: Option<OrderSide>,
    outcome_side: Option<Outcome>,
}

pub async fn create_limit_order(
    State(app_state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
    Json(payload): Json<CreateOrderPayload>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    require_field!(payload.market_id);
    require_field!(payload.price);
    require_field!(payload.quantity);
    require_field!(payload.side);
    require_field!(payload.outcome_side);

    let market_id = payload.market_id.unwrap();

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

    let side = payload.side.unwrap();
    let outcome_side = payload.outcome_side.unwrap();
    let user_id = claims.user_id;
    let quantity = payload.quantity.unwrap();

    let price = payload.price.unwrap();
    if price > 100 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Price must be between 0 and 100"
            }))
            .into_response(),
        ));
    }
    // payload.quantity must be up to 2 decimal places
    let quantity_valid_flg = has_max_two_decimal_places(payload.quantity.unwrap());
    if !quantity_valid_flg {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Quantity must be up to 2 decimal places"
            }))
            .into_response(),
        ));
    }

    // asserting the channel exists (not publishing the message)
    app_state
        .jetstream
        .get_or_create_stream(jetstream::stream::Config {
            // these `ORDER` name does not indicate the operations on orders, instead it indicates that the streams is used by order-service microservice, so don't confuse it with the order name and same for it's topics, all topics are prefixed with `order.`
            name: "ORDER".into(),
            subjects: vec!["order.>".into()],
            ..Default::default()
        })
        .await
        .map_err(|e| {
            log_error!("Failed to create jetstream stream - {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to create jetstream stream"
                }))
                .into_response(),
            )
        })?;

    ///////////////// Verifying user holdings ///////////////////////
    // if trade type is sell then check holdings, else check the user's balance

    if side == OrderSide::SELL {
        let holdings = UserHoldings::get_user_holdings_by_outcome(
            &app_state.pg_pool,
            user_id,
            market_id,
            outcome_side,
        )
        .await
        .map_err(|e| {
            log_error!("Failed to get user holdings - {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to get user holdings"
                }))
                .into_response(),
            )
        })?;

        if holdings.shares <= Decimal::ZERO || holdings.shares < from_f64(quantity) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "You do not have enough shares to sell"
                }))
                .into_response(),
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
                log_error!("Failed to get user - {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Failed to get user"
                    }))
                    .into_response(),
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

        let balance = balance - total_user_locked_funds;
        let required_price = from_u8(price) * from_f64(quantity);
        if balance < Decimal::ZERO || balance < required_price {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "You do not have enough balance to create order"
                }))
                .into_response(),
            ));
        }

        if balance <= required_price {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "You do not have enough balance to buy"
                }))
                .into_response(),
            ));
        }
    }

    ///////////////////////////////////////////////////////////////
    let price = from_u8(price) / dec!(100); // scaling down the price to 2 decimal places (0-100 to 0.00-1.00)

    let order = Order::create_order(
        user_id,
        market_id,
        price, // scaling down the price to 2 decimal places (0-100 to 0.00-1.00)
        from_f64(quantity),
        side,
        outcome_side,
        OrderType::LIMIT,
        &app_state.pg_pool,
    )
    .await
    .map_err(|e| {
        log_error!("Failed to create order - {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to create order"
            }))
            .into_response(),
        )
    })?;

    let mut is_failed = false;

    // pushing the order to the jetstream
    let order_id_str = order.id.to_string().into_bytes();

    let subject = NatsSubjects::OrderCreate;

    app_state
        .jetstream
        .publish(subject.to_string(), order_id_str.into())
        .await
        .map_err(|e| {
            log_error!("Failed to publish order to jetstream - {:?}", e);
            // delete the order from the database if publishing fails
            is_failed = true;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to publish order to jetstream, order will be deleted"
                }))
                .into_response(),
            )
        })?;

    // if the order is failed, delete it from the database
    if is_failed {
        Order::update_order_status(order.id, OrderStatus::CANCELLED, &app_state.pg_pool)
            .await
            .map_err(|e| {
                log_error!("Failed to delete order - {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Failed to update order status to cancelled"
                    }))
                    .into_response(),
                )
            })?;
    }

    log_info!("Order published to jetstream - {:?}", order.id);

    let response = json!({
        "message": "Order created successfully",
        "order" : {
            "id": order.id,
            "user_id": order.user_id,
            "market_id": order.market_id,
            "side": order.side,
            "outcome": order.outcome,
            "price": order.price.to_string(),
            "quantity": order.quantity.to_string(),
            "filled_quantity": order.filled_quantity.to_string(),
            "status": order.status,
        }
    });

    Ok((StatusCode::CREATED, Json(response)))
}

fn from_f64(value: f64) -> Decimal {
    Decimal::from_f64(value)
        .unwrap_or_else(|| panic!("Failed to convert f64 to Decimal: {}", value))
}

fn from_u8(value: u8) -> Decimal {
    Decimal::from_u8(value).unwrap_or_else(|| panic!("Failed to convert u8 to Decimal: {}", value))
}

fn has_max_two_decimal_places(value: f64) -> bool {
    let scaled = (value * 100.0).round();
    (value * 100.0 - scaled).abs() < f64::EPSILON
}
