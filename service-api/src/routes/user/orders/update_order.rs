use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use db_service::schema::{enums::OrderStatus, orders::Order};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::json;
use utility_helpers::{
    message_pack_helper::serialize_to_message_pack,
    nats_helper::{NatsSubjects, types::UpdateOrderMessage},
};
use uuid::Uuid;

use crate::{require_field, state::AppState};

#[derive(Deserialize)]
pub struct UpdateOrderRequestData {
    pub order_id: Option<Uuid>,
    pub new_quantity: Option<Decimal>,
    pub new_price: Option<Decimal>,
}

pub async fn update_order(
    State(app_state): State<AppState>,
    Json(update_order_message): Json<UpdateOrderRequestData>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let order_id = update_order_message.order_id;
    let new_quantity = update_order_message.new_quantity;
    let new_price = update_order_message.new_price;

    require_field!(order_id);
    require_field!(new_quantity);
    require_field!(new_price);

    let order_id = order_id.unwrap();
    let new_quantity = new_quantity.unwrap();
    let new_price = new_price.unwrap();

    let order = Order::find_order_by_id(order_id, &app_state.pg_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})).into_response(),
            )
        })?;

    if order.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Order not found"})).into_response(),
        ));
    }

    let order = order.unwrap();

    if order.quantity == order.filled_quantity {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Order is already fully filled"})).into_response(),
        ));
    }

    if new_quantity <= order.filled_quantity {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "New filled quantity must be greater than current filled quantity"}))
                .into_response(),
        ));
    }

    let update_order_message = UpdateOrderMessage {
        order_id,
        new_quantity,
        new_price,
    };

    let encoded_message = serialize_to_message_pack(&update_order_message).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})).into_response(),
        )
    })?;

    // update order state
    Order::update_order_status(order_id, OrderStatus::PendingUpdate, &app_state.pg_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})).into_response(),
            )
        })?;

    let subject = NatsSubjects::OrderUpdate;
    app_state
        .jetstream
        .publish(subject.to_string(), encoded_message.into())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})).into_response(),
            )
        })?;

    Ok(Json(json!({
        "message": "Order updated request sent successfully",
        "order_id": order_id,
    }))
    .into_response())
}
