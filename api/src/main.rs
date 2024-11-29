use axum::{routing, Router};
use tokio::net::TcpListener;

use anyhow::Context;

pub mod resources;
pub mod testing;
pub mod routes;
pub mod error;

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
        .route("/", routing::get(routes::root))
        .route("/mem", routing::get(routes::mem))
        .route("/cpu", routing::get(routes::cpu))
        .route("/uptime", routing::get(routes::uptime))
        .route("/mem/rt", routing::get(routes::mem_sse))
        .route("/cpu/rt", routing::get(routes::cpu_sse))
        .route("/uptime/rt", routing::get(routes::uptime_sse));

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
