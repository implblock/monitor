use axum::{response::{sse::Event, Sse}, Json};

use monitor_core::probe::Probe;
use std::convert::Infallible;

use tokio_stream::StreamExt;
use futures_util::Stream;

use crate::{
    error::ApiError,
    resources::memory::Memory
};

pub async fn mem_sse() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let x = Memory::stream().map(|x| {
        Event::default().json_data(x)
    })
    .filter_map(|x| x.ok())
    .map(Ok);

    Sse::new(x)
}

pub async fn mem() -> Result<Json<Memory>, ApiError> {
    Ok(Memory::probe().await.map(Json)?)
}
