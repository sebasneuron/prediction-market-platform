use axum::{Router, routing::get};

use crate::state::AppState;

pub mod holdings;
pub mod metadata;
pub mod orders;
pub mod profile;
pub mod trades;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/orders", orders::router())
        .nest("/trades", trades::router())
        .route("/profile", get(profile::get_profile))
        .route("/metadata", get(metadata::get_metadata))
        .route("/holdings", get(holdings::get_user_holdings))
}
