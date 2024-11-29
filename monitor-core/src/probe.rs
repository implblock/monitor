use std::{future::Future, time::Duration};

use futures_util::Stream;
use tokio_stream::StreamExt;

// Something that can be probed--
// Useful for getting information
// from a resource
pub trait Probe {
    fn probe() -> impl Future<Output = Result<Self::Output, Self::Error>>;

    // probes the resource repeatedly, filtering out
    // any failed probe actions
    fn stream() -> impl Stream<Item = Self::Output> {
        futures_util::stream::repeat_with(|| {
            async move {
                tokio::time::sleep(
                    Duration::from_millis(
                        Self::PROBE_DELAY
                    )
                ).await;

                Self::probe().await
            }
        })
        .then(|x| x)
        .filter_map(|x| x.ok())
    }

    const PROBE_DELAY: u64 = 1000 / 12; // MS

    type Output;

    type Error;
}
