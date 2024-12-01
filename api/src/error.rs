use axum::{http::StatusCode, response::IntoResponse};

use crate::resources::{cpu, memory, network, uptime};

pub struct ApiError(anyhow::Error);

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        let err: anyhow::Error = value.into();

        ApiError(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        // TODO
        // make better
        if let Some(e) = self.0.downcast_ref::<cpu::UsageError>() {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{:#?}", e),
            )
            .into_response();
        }

        if let Some(e) = self.0.downcast_ref::<network::Error>() {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{:#?}", e),
            )
            .into_response();
        }

        if let Some(e) = self.0.downcast_ref::<uptime::Error>() {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{:#?}", e),
            )
            .into_response();
        }

        if let Some(e) = self.0.downcast_ref::<memory::Error>() {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{:#?}", e),
            )
            .into_response();
        }

        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{:#?}", self.0)
        )
        .into_response()
    }
}
