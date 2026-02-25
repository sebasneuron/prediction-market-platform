#[macro_export]
macro_rules! require_field {
    ($field:expr) => {
        if $field.is_none() {
            let full = stringify!($field);
            let short = full.split('.').last().unwrap_or(full);
            return Err((
                    axum::http::StatusCode::BAD_REQUEST,
                    axum::Json(serde_json::json!({
                        "message": format!("Missing required field: {}", short),
                    })).into_response(),
            ))
        }
    };
}

#[macro_export]
macro_rules! require_fields_raw_response {
    ($field:expr) => {
        if $field.is_none() {
            let full = stringify!($field);
            let short = full.split('.').last().unwrap_or(full);
            return Err((
                    axum::http::StatusCode::BAD_REQUEST,
                    axum::Json(serde_json::json!({
                        "message": format!("Missing required field: {}", short),
                    })),
            ))
        }
    };
}

#[macro_export]
macro_rules! validate_paginated_fields {
    ($page:expr, $page_size:expr) => {

        let page = $page;
        let page_size = $page_size;

        if page == 0 || page_size == 0 {
            return Err((
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({
                    "message": "Page and page_size must be greater than 0"
                }))
                .into_response(),
            ));
        }
    };
}
