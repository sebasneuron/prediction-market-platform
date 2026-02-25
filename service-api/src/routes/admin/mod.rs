use axum::Router;

use crate::state::AppState;

pub mod markets;

pub fn router() -> Router<AppState> {
    Router::new().nest("/market", markets::market_router())
}
