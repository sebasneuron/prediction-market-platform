use axum::{
    Router,
    routing::{delete, get, patch, post},
};

use crate::state::AppState;

pub mod cancel_order;
pub mod create_limit_order;
pub mod create_market_order;
pub mod get_all_users_orders;
pub mod get_orders_by_markets;
pub mod update_order;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/get", get(get_all_users_orders::get_all_users_orders))
        .route(
            "/create/limit",
            post(create_limit_order::create_limit_order),
        )
        .route(
            "/create/market",
            post(create_market_order::create_limit_order),
        )
        .route(
            "/get/{id}",
            get(get_orders_by_markets::get_user_orders_by_market),
        )
        .route("/cancel/{id}", delete(cancel_order::cancel_order))
        .route("/update", patch(update_order::update_order))
}
