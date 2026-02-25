use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use db_service::schema::{enums::OrderStatus, orders::Order};
use serde_json::json;
use utility_helpers::{log_error, nats_helper::NatsSubjects};
use uuid::Uuid;

use crate::state::AppState;

pub async fn cancel_order(
    Path(id): Path<Uuid>,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let order = Order::find_order_by_id_and_status(id, OrderStatus::OPEN, &app_state.pg_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("Failed to find order: {}", e)
                }))
                .into_response(),
            )
        })?;

    if order.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Order not found, or it is not open."
            }))
            .into_response(),
        ));
    }

    Order::update_order_status(id, OrderStatus::PendingCancel, &app_state.pg_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("Failed to update order status: {}", e)
                }))
                .into_response(),
            )
        })?;

    // assertion is not needed, as it's already checked while creating the order

    // publishing the order to the delete order queue
    let order_id_str = id.to_string().into_bytes();
    let subject = NatsSubjects::OrderCancel;

    app_state
        .jetstream
        .publish(subject.to_string(), order_id_str.into())
        .await
        .map_err(|e| {
            log_error!("Failed to publish order to jetstream - {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to publish order to jetstream, order will be deleted"
                }))
                .into_response(),
            )
        })?;

    Ok(Json(json!({
        "message": "Order cancellation request sent successfully",
        "success": true,
    })))
}
