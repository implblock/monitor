use axum::{
    response::{
        sse::Event,
        Sse,
    },
    Json,
};

use std::convert::Infallible;

use tokio_stream::StreamExt;
use monitor::probe::Probe;
use futures_util::Stream;

use crate::{
    resources::cpu::Cpu,
    error::ApiError,
};

pub async fn cpu_sse() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let s = Cpu::stream().map(|x| {
        Event::default().json_data(x)
    })
    .filter_map(|x| x.map(Ok).ok());

    Sse::new(s)
}

pub async fn cpu() -> Result<Json<Cpu>, ApiError> {
    Ok(Cpu::probe().await.map(Json)?)
}
