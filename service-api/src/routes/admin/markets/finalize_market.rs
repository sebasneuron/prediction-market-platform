use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use db_service::schema::{
    enums::{MarketStatus, Outcome},
    market::Market,
};
use serde::Deserialize;
use serde_json::json;
use utility_helpers::{log_error, nats_helper::NatsSubjects};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct FinalizeMarketRequest {
    pub market_id: Uuid,
    pub final_outcome: Outcome,
}

pub async fn finalize_market(
    State(state): State<AppState>,
    Json(payload): Json<FinalizeMarketRequest>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let market_id = payload.market_id;
    let final_outcome = payload.final_outcome;

    let market = Market::get_market_by_id(&state.pg_pool, &market_id)
        .await
        .map_err(|e| {
            log_error!("Failed to get market by ID: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to get market by ID"
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

    let market = market.unwrap();
    if market.status != MarketStatus::OPEN {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Market is not open"
            }))
            .into_response(),
        ));
    }

    Market::settle_market(&state.pg_pool, &market_id, final_outcome)
        .await
        .map_err(|e| {
            log_error!("Failed to settle market: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to settle market"
                }))
                .into_response(),
            )
        })?;

    // push message to NATS
    let nats_subject = NatsSubjects::FinalizeMarket.to_string();
    let message = market_id.to_string().into_bytes();

    state
        .jetstream
        .publish(nats_subject, message.into())
        .await
        .map_err(|e| {
            log_error!("Failed to publish NATS message: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to publish NATS message"
                }))
                .into_response(),
            )
        })?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "Message": "Market finalized successfully",
        })),
    ))
}
