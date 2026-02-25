use std::sync::Arc;

use axum::{
    Json,
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
};
use serde_json::json;

use crate::state::AppState;

pub async fn validate_jwt(
    State(app_state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, impl IntoResponse)> {
    let missing_token_error = (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "error": "Missing auth token"
        })),
    );
    let invalid_token_error = (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": "Invalid token"
        })),
    );
    let auth_header = req.headers().get("Authorization");
    let token = match auth_header {
        Some(header) => header
            .to_str()
            .ok()
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or(missing_token_error)?,
        None => return Err(missing_token_error),
    };

    let claims = app_state
        .auth_service
        .verify_session_token(token)
        .map_err(|_| invalid_token_error)?;

    // bloom filter check
    let user_id = claims.user_id;

    if !app_state.bloom_filter.contains(&user_id) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "User not found"
            })),
        ));
    }

    let mut req = req;
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}
