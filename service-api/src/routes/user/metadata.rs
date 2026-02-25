use auth_service::types::SessionTokenClaims;
use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use db_service::schema::users::User;
use serde_json::json;

use crate::state::AppState;

pub async fn get_metadata(
    State(app_state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let user_id = claims.user_id;

    let user_profile_insight = User::get_user_metadata(&app_state.pg_pool, user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to fetch user metadata",
                    "details": e.to_string()
                }))
                .into_response(),
            )
        })?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "user_id": user_id,
            "profile_insight": user_profile_insight
        })),
    ))
}
