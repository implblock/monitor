use axum::{routing, Router};
use tokio::net::TcpListener;

use anyhow::Context;

pub mod health_check;
pub mod routes;

// for simple empty results
type Any = anyhow::Result<()>;

const PORT: u16 = 6942;

#[tokio::main]
async fn main() -> Any {
    let addr = std::env::var("ADDR").unwrap_or(
        // the default shouldn't be loopback
        format!("0.0.0.0:{}", PORT)
    );

    let socket = TcpListener::bind(&addr).await
        .with_context(|| "connecting to socket")?;

    let router = Router::new()
        .route("/", routing::get(routes::root));

    tracing::info!(
        "now serving on {}", addr,
    );

    axum::serve(
        socket,
        router,
    ).await
    .with_context(|| "serving")?;

    Ok(())
}
