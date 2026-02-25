use axum::{Router, routing::post};

use crate::state::AppState;

pub mod create_market;
pub mod finalize_market;
pub mod initialize_market;

pub fn market_router() -> Router<AppState> {
    Router::new()
        .route("/create", post(create_market::create_new_market))
        .route("/initialize", post(initialize_market::initialize_market))
        .route("/finalize", post(finalize_market::finalize_market))
}
