use auth_service::types::SessionTokenClaims;
use axum::{
    Extension, Json,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use db_service::schema::{enums::OrderStatus, orders::Order};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utility_helpers::log_error;

use crate::{require_field, state::AppState, validate_paginated_fields};

#[derive(Deserialize, Serialize, Debug)]
pub struct QueryParams {
    page: Option<u32>,
    page_size: Option<u32>,
    status: Option<String>, // Optional field to filter by order status
}

pub async fn get_all_users_orders(
    State(app_state): State<AppState>,
    Query(params): Query<QueryParams>,
    Extension(claims): Extension<SessionTokenClaims>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let user_id = claims.user_id;
    let status = params.status.as_deref().unwrap_or("open");

    require_field!(params.page);
    require_field!(params.page_size);

    let page = params.page.unwrap();
    let page_size = params.page_size.unwrap();
    let order_status = match status.to_lowercase().as_str() {
        "open" => OrderStatus::OPEN,
        "cancelled" => OrderStatus::CANCELLED,
        "filled" => OrderStatus::FILLED,
        "expired" => OrderStatus::EXPIRED,
        "pending_update" => OrderStatus::PendingUpdate,
        "pending_cancel" => OrderStatus::PendingCancel,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"message": "Invalid order status"})).into_response(),
            ));
        }
    };

    validate_paginated_fields!(page, page_size);

    let (user_orders, total_page) = Order::get_user_orders_by_paginated(
        &app_state.pg_pool,
        user_id,
        order_status,
        page,
        page_size,
    )
    .await
    .map_err(|e| {
        log_error!("Failed to fetch user orders {e:?}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": "Failed to fetch user orders"})).into_response(),
        )
    })?;

    Ok(Json(json!({
        "orders": user_orders,
        "page": page,
        "page_size": page_size,
        "total_pages": total_page,
    }))
    .into_response())
}
