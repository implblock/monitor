use axum::{http::StatusCode, response::IntoResponse};

pub struct ApiError(StatusCode, anyhow::Error);

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        ApiError(StatusCode::INTERNAL_SERVER_ERROR, value.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            self.0,
            self.1.to_string(),
        )
        .into_response()
    }
}
