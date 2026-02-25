use axum::{
    Json, Router,
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::json;
use std::sync::Arc;

use crate::state::AppState;
use crate::utils::middleware as custom_middleware;

pub mod admin;
pub mod login;
pub mod user;

async fn default_home_route() -> (StatusCode, impl IntoResponse) {
    let welcome_message = json!({
        "message": "Welcome to the Polymarket clone service API!"
    });
    (StatusCode::OK, Json(welcome_message))
}

pub fn router(app_state: AppState) -> Router<AppState> {
    let app_state = Arc::new(app_state.clone());
    let user_routes = user::router().layer(middleware::from_fn_with_state(
        app_state,
        custom_middleware::validate_jwt,
    ));

    let admin_routes = admin::router(); // for now without middleware

    Router::new()
        .route("/", get(default_home_route))
        .route("/login", post(login::oauth_login))
        .nest("/user", user_routes)
        .nest("/admin", admin_routes)
}
