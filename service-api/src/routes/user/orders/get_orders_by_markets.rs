use auth_service::types::SessionTokenClaims;
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use db_service::schema::{enums::OrderStatus, orders::Order, user_holdings::UserHoldings};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utility_helpers::log_error;
use uuid::Uuid;

use crate::{require_field, state::AppState, validate_paginated_fields};

#[derive(Deserialize, Serialize, Debug)]
pub struct QueryParams {
    page: Option<u32>,
    page_size: Option<u32>,
    status: Option<String>, // Optional field to filter by order status
}

pub async fn get_user_orders_by_market(
    State(app_state): State<AppState>,
    Query(params): Query<QueryParams>,
    Extension(claims): Extension<SessionTokenClaims>,
    Path(market_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let user_id = claims.user_id;
    let status = params.status.as_deref().unwrap_or("open");

    require_field!(params.page);
    require_field!(params.page_size);

    let page = params.page.unwrap();
    let page_size = params.page_size.unwrap();

    let order_status = match status.to_lowercase().as_str() {
        "open" => Some(OrderStatus::OPEN),
        "cancelled" => Some(OrderStatus::CANCELLED),
        "filled" => Some(OrderStatus::FILLED),
        "expired" => Some(OrderStatus::EXPIRED),
        "pending_update" => Some(OrderStatus::PendingUpdate),
        "pending_cancel" => Some(OrderStatus::PendingCancel),
        "all" => None,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"message": "Invalid order status"})).into_response(),
            ));
        }
    };

    validate_paginated_fields!(page, page_size);

    let user_orders_future = Order::get_user_orders_by_market_paginated(
        &app_state.pg_pool,
        user_id,
        market_id,
        page,
        page_size,
        order_status,
    );
    let user_holdings_future =
        UserHoldings::get_user_holdings_sum_both_outcome(&app_state.pg_pool, user_id, market_id);

    let (user_orders_result, user_holdings_result) =
        tokio::join!(user_orders_future, user_holdings_future,);

    let (user_orders, total_page) = user_orders_result.map_err(|e| {
        log_error!("Failed to fetch user orders {e:?}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": "Failed to fetch user orders"})).into_response(),
        )
    })?;
    let (yes_holdings, no_holdings) = user_holdings_result.map_err(|e| {
        log_error!("Failed to fetch user holdings {e:?}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": "Failed to fetch user holdings"})).into_response(),
        )
    })?;

    Ok(Json(json!({
        "holdings": {
            "yes": yes_holdings,
            "no": no_holdings,
        },
        "orders": user_orders,
        "page": page,
        "page_size": page_size,
        "total_pages": total_page,
    }))
    .into_response())
}
