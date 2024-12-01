use std::convert::Infallible;

use axum::{response::{sse::Event, Sse}, Json};
use futures_util::Stream;
use monitor::probe::Probe;
use tokio_stream::StreamExt;

use crate::{error::ApiError, resources::uptime::Uptime};

pub async fn uptime_sse() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let s = Uptime::stream().map(|x| {
        Event::default().json_data(x)
    })
    .filter_map(|x| x.ok())
    .map(Ok);

    Sse::new(s)
}

pub async fn uptime() -> Result<Json<Uptime>, ApiError> {
    Ok(Uptime::probe().await.map(Json)?)
}
