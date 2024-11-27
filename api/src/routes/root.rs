use axum::{http::StatusCode, response::IntoResponse};

pub async fn root() -> impl IntoResponse {
    StatusCode::OK
}
