use auth_service::types::SessionTokenClaims;
use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use db_service::schema::users::User;
use serde_json::json;
use utility_helpers::{log_error, redis::keys::RedisKey};

use crate::state::AppState;

pub async fn get_profile(
    State(app_state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let user_id = claims.user_id;

    let user_key = RedisKey::User(user_id);
    let user = app_state
        .redis_helper
        .get_or_set_cache(
            user_key,
            || async { Ok(User::get_user_by_id(&app_state.pg_pool, user_id).await?) },
            Some(20), // Cache for 20 seconds
        )
        .await
        .map_err(|err| {
            log_error!("Failed to retrieve user profile: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to retrieve user profile"
                }))
                .into_response(),
            )
        })?;

    let response = json!({
        "email": user.email,
        "name": user.name,
        "avatar": user.avatar,
        "public_key": user.public_key,
        "balance": user.balance,
    });

    Ok((StatusCode::OK, Json(response)))
}
