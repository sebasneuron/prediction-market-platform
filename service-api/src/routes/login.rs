use auth_service::types::AuthenticateUserError;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utility_helpers::log_error;

use crate::{require_field, state::AppState, utils::types::ReturnType};

#[derive(Deserialize, Serialize)]
pub struct LoginRequest {
    id_token: Option<String>,
}

pub async fn oauth_login(
    State(app_state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<ReturnType, ReturnType> {
    require_field!(payload.id_token);
    let id_token = payload.id_token.as_ref().unwrap(); // already verified by require_field!

    let (user_id, session_token, is_new_user) = app_state
        .auth_service
        .authenticate_user(id_token)
        .await
        .map_err(|e| {
            log_error!("Failed to authenticate user: {:?}", e);
            match e {
                AuthenticateUserError::InvalidToken => (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "Invalid token"})).into_response(),
                ),
                AuthenticateUserError::FailedToInsertUser => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to insert user"})).into_response(),
                ),
                AuthenticateUserError::FailedToGenerateSessionToken => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to generate session token"})).into_response(),
                ),
            }
        })?;

    // update bloom filter with new user id
    app_state.bloom_filter.insert(&user_id);

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": if is_new_user {
                "User created successfully"
            } else {
                "User logged in successfully"
            },
            "userId": user_id,
            "sessionToken": session_token,
            "success": true
        }))
        .into_response(),
    ))
}
