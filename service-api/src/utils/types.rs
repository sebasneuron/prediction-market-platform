use axum::{http::StatusCode, response::Response};
use serde::Deserialize;

pub type ReturnType = (StatusCode, Response);

#[derive(Deserialize, Clone, Debug)]
pub struct PaginationRequestQuery {
    pub page: u64,
    #[serde(rename = "pageSize")]
    pub page_size: u64,
}
