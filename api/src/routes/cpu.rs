use std::convert::Infallible;

use axum::{response::{sse::Event, Sse}, Json};
use monitor_core::probe::Probe;
use futures_util::Stream;
use tokio_stream::StreamExt;

use crate::{error::ApiError, resources::cpu::Cpu};

pub async fn cpu_sse() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let s = Cpu::stream().map(|x| {
        Event::default().json_data(x)
    })
    .filter_map(|x| x.map(Ok).ok());

    Sse::new(s)
}

pub async fn cpu() -> Result<Json<Cpu>, ApiError> {
    Ok(Json(Cpu::probe().await?))
}
