use axum::{Router, routing::get};

use crate::state::AppState;

mod get_user_trades;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_user_trades::get_user_trades))
}
