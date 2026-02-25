use auth_service::types::SessionTokenClaims;
use axum::{
    Extension, Json,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use db_service::schema::user_holdings::UserHoldings;
use serde_json::json;
use utility_helpers::log_error;

use crate::{state::AppState, utils::types::PaginationRequestQuery, validate_paginated_fields};

pub async fn get_user_holdings(
    State(state): State<AppState>,
    Query(params): Query<PaginationRequestQuery>,
    Extension(claims): Extension<SessionTokenClaims>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let user_id = claims.user_id;

    validate_paginated_fields!(params.page, params.page_size);

    let holdings = UserHoldings::get_user_holdings_by_market_paginated(
        user_id,
        params.page,
        params.page_size,
        &state.pg_pool,
    )
    .await
    .map_err(|e| {
        log_error!("Failed to fetch user holdings {e:?}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": "Failed to fetch user holdings"})).into_response(),
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "User holdings fetched successfully",
            "data": {
                "holdings": holdings.items,
                "page_info": holdings.page_info
            }
        })),
    ))
}
