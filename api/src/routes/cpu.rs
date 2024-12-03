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
    error::ApiError, resources::cpu::{Core, Cores, Cpu, Info, Usage}
};

pub async fn cpu_cores_sse() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let s = Cores::stream().map(|x| {
        Event::default().json_data(x)
    })
    .filter_map(|x| x.map(Ok).ok());

    Sse::new(s)
}

pub async fn cpu_usage_sse() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let s = Usage::stream().map(|x| {
        Event::default().json_data(x)
    })
    .filter_map(|x| x.map(Ok).ok());

    Sse::new(s)
}

pub async fn cpu_info_sse() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let s = Info::stream().map(|x| {
        Event::default().json_data(x)
    })
    .filter_map(|x| x.map(Ok).ok());

    Sse::new(s)
}

pub async fn cpu_sse() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let s = Cpu::stream().map(|x| {
        Event::default().json_data(x)
    })
    .filter_map(|x| x.map(Ok).ok());

    Sse::new(s)
}

pub async fn cpu_cores() -> Result<Json<Vec<Core>>, ApiError> {
    Ok(Cores::probe().await.map(Json)?)
}

pub async fn cpu_usage() -> Result<Json<Usage>, ApiError> {
    Ok(Usage::probe().await.map(Json)?)
}

pub async fn cpu_info() -> Result<Json<Info>, ApiError> {
    Ok(Info::probe().await.map(Json)?)
}

pub async fn cpu() -> Result<Json<Cpu>, ApiError> {
    Ok(Cpu::probe().await.map(Json)?)
}
