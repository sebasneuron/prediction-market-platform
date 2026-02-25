use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use db_service::schema::market::Market;
use rust_decimal::{Decimal, prelude::FromPrimitive};
use serde_json::json;
use sqlx::types::chrono::{self, DateTime};
use utility_helpers::log_error;

use crate::{require_fields_raw_response, state::AppState};

#[derive(serde::Deserialize)]
pub struct CreateMarketRequest {
    name: Option<String>,
    description: Option<String>,
    logo: Option<String>,
    liquidity_b: Option<f64>,
    market_expiry: Option<String>,
}

// Add market expiry in db
pub async fn create_new_market(
    State(state): State<AppState>,
    Json(payload): Json<CreateMarketRequest>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    require_fields_raw_response!(payload.name);
    require_fields_raw_response!(payload.description);
    require_fields_raw_response!(payload.logo);
    require_fields_raw_response!(payload.liquidity_b);
    require_fields_raw_response!(payload.market_expiry);

    let liquidity_b = payload.liquidity_b.unwrap();
    let name = payload.name.unwrap();
    let description = payload.description.unwrap();
    let logo = payload.logo.unwrap();
    let market_expiry = payload.market_expiry.unwrap();

    let liquidity_b = Decimal::from_f64(liquidity_b).ok_or_else(|| {
        log_error!("Invalid liquidity_b value: {}", liquidity_b);
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid liquidity_b value"
            })),
        )
    })?;
    let date_time = DateTime::parse_from_rfc3339(&market_expiry).map_err(|e| {
        log_error!("Invalid market_expiry format: {} due to {e}", market_expiry);
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid market_expiry format"
            })),
        )
    })?;

    // if date time is in the past, return an error
    if date_time < chrono::Utc::now() {
        log_error!("Market expiry date is in the past: {}", market_expiry);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Market expiry date cannot be in the past"
            })),
        ));
    }

    let market_expiry = date_time.naive_utc();

    let market = Market::create_new_market(
        name,
        description,
        logo,
        liquidity_b,
        market_expiry,
        &state.pg_pool,
    )
    .await
    .map_err(|e| {
        log_error!("Error creating market: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to create market"
            })),
        )
    })?;

    let response = json!({
        "message": "Market created successfully",
        "market": {
            "id": market.id,
            "name": market.name,
            "description": market.description,
            "logo": market.logo,
            "liquidity_b": market.liquidity_b,
        }
    });
    Ok((StatusCode::CREATED, Json(response)).into_response())
}
