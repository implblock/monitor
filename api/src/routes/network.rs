use std::convert::Infallible;

use axum::{response::{sse::Event, Sse}, Json};
use futures_util::Stream;
use monitor::probe::Probe;
use tokio_stream::StreamExt;

use crate::{error::ApiError, resources::network::Network};

pub async fn network_sse() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let s = Network::stream().map(|x| {
        Event::default().json_data(x)
    })
    .filter_map(|x| x.ok())
    .map(Ok);

    Sse::new(s)
}

pub async fn network() -> Result<Json<Network>, ApiError> {
    Ok(Network::probe().await.map(Json)?)
}
